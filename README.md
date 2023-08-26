# TRustAss

T-Rust Assistant: A Distributed file synchronizer using the vector-time-pairs algorithm.

### Features

This project has the following **file synchronization** features:

- Use TRA algorithm to synchronize files, **requiring no central server**. (Though in this project I build a central server for sending sync messages to all replicas.)
- Detect all kinds of conflicts **without any false positives**.
- Support **the deletion synchronization** of files and directories.
- Support **manual conflict resolution**, and once resolved, the conflict will never occur again.
- Support **partial synchronization**, which means any subdirectory can be synchronized.
- Strictly **identify the set of files that need to be synchronized**, and only synchronize these files.

Also, this project has the following **extra features**:

- Totally **written in Rust**, along with the **`tokio` asynchronous runtime**, which is highly efficient and supports numerous concurrent synchronization tasks at the same time.
- Use **`tonic` gRPC framework** to implement the communication between replicas and the central server, supporting asynchronous streaming in both directions.
- Use **`rsync` algorithm** to synchronize files, which means only the differences between files will be transmitted.
- Use **`inotify` to monitor file changes**, which means the local modifications will be detected immediately and the metadata of files will be updated in time.

### Implementation Specifications

- Multiple replicas are simulated by multiple threads.
- Each replica has a unique ID and a random port number, communicating with each other through socket connections.
- Each replica's root directory is located in the `./tmp/replica-<id>` directory by default.

### Usage

Just `cargo run` to start the distributed synchronizer CLI.

##### Types of Information

A local watcher has been added or removed.

```
👀 Local Watch: <path> <action>
```

A local modification/creation/deletion has been detected.

```
📢 Local <event>: <path>
```

A new synchronization request has been received.

```
🔄 Sync Request : replica-<id>(<port>) -> replica-<id>(<port>) : <path>
```

Successfully synchronized a file.

```
✔ Sync <situation> : <path> (<reason>)
```

There is a conflict that needs a resolution.

```
🔧 Sync Conflict : <path>
```

Some warnings or errors occur.

```
⚠️ Warning : <message>
❌ Error : <message>
```

##### Tree Command

The command `tree <id>` will show the tree-like structure of the replica with the given id. Normally there are modification timestamps and synchronization timestamps just after the file names, displayed in yellow and green respectively.

```bash
(tra) ❯ tree 1
replica-1 <m-time> <s-time>  
└── dir1 <m-time> <s-time>
    ├── a.cpp <m-time> <s-time>
    ├── b.cpp <m-time> <s-time>
    └── dir2 <m-time> <s-time> 
        └── c.cpp <m-time> <s-time>
```

##### Sync Command

```bash
(tra) ❯ sync 1 2 dir1
🔄 Sync Request : replica-1(56127) -> replica-2(59171), path = "dir1"
✔ Sync Skip : "./tmp/replica-2/dir1" (newer)
```

##### Resolution Selection

When conflicts occur, the program will ask you to select a resolution. Use the arrow keys to move the cursor and press Enter to select.

```bash
  use the local version
  use the remote version
❯ handle manually
```

When you select `handle manually`, the program will open the default editor to let you edit the different versions of the file. After you save and exit the editor, the program will automatically resolve the conflict and continue the synchronization.

##### Exit Command

Just type `exit` to exit the program.

```bash
(tra) ❯ exit
Shutting down the command line interface ...
```

### Attention

- Though each pattern of synchronization has been tested, there could still be some bugs in the synchronization process, so please backup your files before using this program.
- When a folder or a file is being synchronized, do not modify it. Otherwise, the synchronization may fail.
- Some IDEs or editors may create temporary files when editing files, which brings some confusion to the original timestamp vector mechanism.
- The `inotify` event watcher may have some critical delays, which bring false positives to the local modification detection.

### References

- [File Synchronization with Vector Time Pairs](http://publications.csail.mit.edu/tmp/MIT-CSAIL-TR-2005-014.pdf)
- [Optimistic Replication Using Vector Time Pairs](https://pdos.csail.mit.edu/archive/6.824-2004/papers/tra.pdf)
- [How to Build a File Synchronizer](http://web.mit.edu/6.033/2005/wwwdocs/papers/unisonimpl.pdf)