# Lock ordering enforcement at compile time

This library contains types and traits to ensure, at compile time, that locks
are acquired in the correct order.

## Credits

Inspired by [Fuchsia](https://fuchsia.dev)'s [lock-ordering] and [lock-sequence] libraries.

[lock-sequence]: https://cs.opensource.google/fuchsia/fuchsia/+/c9ba0365b5ad18b1e45dbc9cd910bbf578830cfc:src/starnix/lib/lock-sequence/
[lock-ordering]: https://cs.opensource.google/fuchsia/fuchsia/+/c9ba0365b5ad18b1e45dbc9cd910bbf578830cfc:src/connectivity/network/netstack3/core/lock-order/