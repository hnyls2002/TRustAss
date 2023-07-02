### Basic Framework

- `tra` : a central application managing the whole system.
- `trasrv` : two servers to synchronize the data.

#### Worker threads

In order to have many RPCs in flight and thus use the network well, `tra` is structured as a large number of worker threads each directing the synchronization of a single file or directory.

Here maybe we use coroutines in Rust to implement the worker threads.
