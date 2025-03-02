//! A toy chat server.
//!
//! A simple chat server that allows users to send messages to one another. The server supports
//! - creating users
//! - sending messages from one user to another
//! - notifying the sender when its message is delivered
//! - waiting for new messages (via HTTP long-poll)
//!
//! See the [`main`] entry point for HTTP endpoints.

/// Unique identifier for a user
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct UserId(u32);

/// A text message sent from one user to another.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TextMessage {
    sender: UserId,
    body: String,
}

/// A message received by a user.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub enum ReceivedMessage {
    /// Text message sent from another user
    Text(TextMessage),
    /// Delivery receipt for a previously-sent [`TextMessage`].
    DeliveryReceipt { original_recipient: UserId },
}

mod state {
    //! The state held by the server.
    //!
    //! Putting this in a separate module requires that accesses be done through
    //! the public methods which enforce lock ordering, as opposed to via direct
    //! field access.

    use lock_ordering::lock::AsyncMutexLockLevel;
    use lock_ordering::relation::LockBefore;
    use lock_ordering::{LockLevel, LockedAt, MutualExclusion};
    use std::collections::HashMap;
    use std::ops::{Deref, DerefMut};
    use std::sync::Arc;
    use tokio::sync::{Mutex, MutexGuard};

    use super::{ReceivedMessage, UserId};

    /// Top-level state object for the entire server.
    #[derive(Debug, Default)]
    pub struct ServerState {
        users: Mutex<UserTable>,
    }

    /// State for users.
    #[derive(Debug, Default)]
    pub struct UserTable {
        /// Mapping from [`UserId`] to [`UserState`].
        ///
        /// The latter is held as an `Arc` so that it can be copied and accessed
        /// after the coarse-grained lock for the table is dropped.
        users: HashMap<UserId, Arc<UserState>>,
        next_id: u32,
    }

    /// State for a single user
    #[derive(Debug, Default)]
    pub struct UserState {
        /// Incoming messages for the user that haven't been acknowledged as
        /// delivered.
        mailbox: Mutex<Queue<ReceivedMessage>>,
        new_message: tokio::sync::Notify,
    }

    type Queue<T> = std::collections::VecDeque<T>;

    impl ServerState {
        /// Returns a reference to the [`UserTable`].
        pub async fn users<'s>(
            &'s self,
            locked: &'s mut LockedAt<'_, impl LockBefore<lock_level::UserTable>>,
        ) -> (
            impl DerefMut<Target = UserTable> + 's,
            LockedAt<'s, lock_level::UserTable>,
        ) {
            let (locked, table) = locked
                .wait_for_lock::<lock_level::UserTable>(&self.users)
                .await;
            (table, locked)
        }

        /// Creates a new user and returns its ID.
        pub async fn create_user<'s>(
            &'s self,
            locked: &'s mut LockedAt<'_, impl LockBefore<lock_level::UserTable>>,
        ) -> Option<UserId> {
            let mut table = locked.wait_lock::<lock_level::UserTable>(&self.users).await;

            let id = UserId(table.next_id);
            table.next_id = table.next_id.checked_add(1)?;

            table.users.insert(id, UserState::default().into());
            Some(id)
        }
    }

    impl UserTable {
        /// Gets the state for an individual user, keyed by ID.
        ///
        /// No [`LockedAt`] is needed because the state is already locked.
        pub fn user_state<'s>(
            &'s self,
            id: &'s UserId,
        ) -> Option<impl Deref<Target = Arc<UserState>> + 's> {
            self.users.get(id)
        }
    }

    impl UserState {
        /// Gets the user's not-yet-acknowledged-as-delivered messages.
        pub async fn messages<'s>(
            &'s self,
            locked: &'s mut LockedAt<'_, impl LockBefore<lock_level::UserMailbox>>,
        ) -> impl Deref<Target = Queue<ReceivedMessage>> + 's {
            locked.wait_lock(&self.mailbox).await
        }

        /// Takes some messages from the beginning of the user's message queue.
        ///
        /// At most `max_count` of the oldest messages in the queue will be
        /// removed. All removed messages are returned.
        pub async fn take_messages<'s>(
            &'s self,
            locked: &'s mut LockedAt<'_, impl LockBefore<lock_level::UserMailbox>>,
            max_count: usize,
        ) -> Vec<ReceivedMessage> {
            let mut messages = locked.wait_lock(&self.mailbox).await;
            let max_count = max_count.min(messages.len());
            messages.drain(0..max_count).collect()
        }

        /// Waits for a message to be delivered, then returns it.
        ///
        /// Returns a message that was delivered after the beginning of this call.
        /// The returned message might not be the only message that was delivered
        /// since the beginning of this call.
        pub async fn next_messsage<'s, L: LockBefore<lock_level::UserMailbox>>(
            &'s self,
            locked: &'s mut LockedAt<'_, L>,
        ) -> impl Deref<Target = ReceivedMessage> + 's {
            loop {
                let () = self.new_message.notified().await;

                // Work around a limitation of the borrow checker: in one branch below we
                // return a value referencing `locked` and in the other we drop the value
                // that borrows from `locked`. This won't be necessary once the new Polonius
                // borrow checker is stabilized.
                let locked: &mut LockedAt<'_, L> = unsafe { &mut *(locked as *mut _) };

                let guard = locked.wait_lock(&self.mailbox).await;
                if let Ok(last) =
                    MutexGuard::try_map(guard, |messages| messages.iter_mut().next_back())
                {
                    return last;
                }
            }
        }

        /// Delivers a message to the users message queue.
        pub async fn deliver_message<'s>(
            &'s self,
            message: ReceivedMessage,
            locked: &'s mut LockedAt<'_, impl LockBefore<lock_level::UserMailbox>>,
        ) {
            let mut guard = locked.wait_lock(&self.mailbox).await;
            guard.push_back(message);
            self.new_message.notify_waiters();
        }
    }

    mod lock_level {
        use lock_ordering::relation::LockAfter;
        use lock_ordering::{impl_transitive_lock_order, Unlocked};

        /// Lock level corresponding to the coarse-grained user table.
        pub enum UserTable {}

        /// Lock level for an individual user's message queue.
        pub enum UserMailbox {}

        /// The coarse-grained table lock cannot be acquired while an individual
        /// user's lock is held.
        impl LockAfter<Unlocked> for UserTable {}
        impl LockAfter<UserTable> for UserMailbox {}
        impl_transitive_lock_order!(UserTable => UserMailbox);
    }

    impl LockLevel for lock_level::UserTable {
        type Method = MutualExclusion;
    }
    impl AsyncMutexLockLevel for lock_level::UserTable {
        type Mutex = Mutex<UserTable>;
    }
    impl LockLevel for lock_level::UserMailbox {
        type Method = MutualExclusion;
    }
    impl AsyncMutexLockLevel for lock_level::UserMailbox {
        type Mutex = Mutex<Queue<ReceivedMessage>>;
    }
}

mod server {
    use std::sync::Arc;

    use axum::extract::{Json, Path, State};

    use super::state::ServerState;
    use super::{ReceivedMessage, TextMessage, UserId};

    pub async fn create_user(
        State(state): State<Arc<ServerState>>,
    ) -> Result<Json<UserId>, axum::http::StatusCode> {
        let mut locked = lock_ordering::LockedAt::new();
        let Some(user_id) = state.create_user(&mut locked).await else {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        };

        Ok(user_id.into())
    }

    pub async fn get_messages(
        State(state): State<Arc<ServerState>>,
        Path(user_id): Path<UserId>,
    ) -> Json<Vec<ReceivedMessage>> {
        let mut locked = lock_ordering::LockedAt::new();
        let (users, mut locked) = state.users(&mut locked).await;
        let Some(user) = users.user_state(&user_id) else {
            return vec![].into();
        };

        let mailbox = (&*user).messages(&mut locked).await;
        Json(mailbox.iter().cloned().collect())
    }

    pub async fn wait_for_message(
        State(state): State<Arc<ServerState>>,
        Path(user_id): Path<UserId>,
    ) -> Result<Json<ReceivedMessage>, axum::http::StatusCode> {
        let mut locked = lock_ordering::LockedAt::new();

        let user_state = {
            let (users, _locked) = state.users(&mut locked).await;
            let Some(user) = users.user_state(&user_id) else {
                return Err(axum::http::StatusCode::NOT_FOUND);
            };

            // Get a handle to the state for the user so the top-level lock can be
            // released.
            Arc::clone(&user)
        };

        let message = user_state.next_messsage(&mut locked).await;
        Ok(Json(ReceivedMessage::clone(&*message)))
    }

    pub async fn acknowledge_messages(
        State(state): State<Arc<ServerState>>,
        Path(user_id): Path<UserId>,
        Json(count): Json<usize>,
    ) {
        let mut locked = lock_ordering::LockedAt::new();
        let (users, mut locked) = state.users(&mut locked).await;

        let delivered = {
            let Some(user) = users.user_state(&user_id) else {
                return;
            };

            (&*user).take_messages(&mut locked, count).await
        };

        for message in delivered {
            let ReceivedMessage::Text(TextMessage { sender, body: _ }) = message else {
                continue;
            };

            let Some(user) = users.user_state(&sender) else {
                continue;
            };
            user.deliver_message(
                ReceivedMessage::DeliveryReceipt {
                    original_recipient: user_id,
                },
                &mut locked,
            )
            .await
        }
    }

    pub async fn send_message(
        State(state): State<Arc<ServerState>>,
        Path(user_id): Path<UserId>,
        Json(message): Json<TextMessage>,
    ) -> axum::http::StatusCode {
        let mut locked = lock_ordering::LockedAt::new();
        // Check that the sending user exists.
        let (users, mut locked) = state.users(&mut locked).await;
        if users.user_state(&message.sender).is_none() {
            return axum::http::StatusCode::UNAUTHORIZED;
        };

        let Some(user) = users.user_state(&user_id) else {
            return axum::http::StatusCode::NOT_FOUND;
        };

        user.deliver_message(ReceivedMessage::Text(message), &mut locked)
            .await;

        axum::http::StatusCode::OK
    }
}

#[tokio::main]
async fn main() {
    use std::net::{Ipv6Addr, SocketAddr};
    use std::str::FromStr as _;

    use axum::routing::*;

    let mut args = std::env::args().skip(1);

    let address = args
        .next()
        .map(|address| {
            SocketAddr::from_str(&address).expect("first argument must be a valid address:port")
        })
        .unwrap_or((Ipv6Addr::UNSPECIFIED, 3000).into());

    let state = std::sync::Arc::new(crate::state::ServerState::default());
    let app = axum::Router::new()
        .route("/user/", post(server::create_user))
        .route("/user/{id}/", get(server::get_messages))
        .route("/user/{id}/", post(server::send_message))
        .route("/user/{id}/", delete(server::acknowledge_messages))
        .route("/user/{id}/wait", get(server::wait_for_message))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
