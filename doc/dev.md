### 基本框架

- `(cen)tra`
  - 中心化的服务器，用于启动不同线程上面的replica同步服务
  - 控制流的传输
  - 测试命令的传输：所有的修改和同步操作由`centra`来统一发送给不同的`reptra`
- `(rep)tra`
  - 接受同步的命令
  - 使用RPC完成fetch patch的操作
- [ ] 支持不同的`reptra`直接使用命令行来同步

### Reptra Emulation

- 采用不同的线程
- 本地随即分配可用端口，然后将端口发送给`centra`，`centra`用不同的端口来区分不同的`reptra`

###  Rsync Algorithm

- Rust 的 `fast_rsync` 库
- 一次RPC
  - A : send signature
  - B : diff with the data, send the patch back
  - A : receive the patch, apply the patch

### Test

- 外部的 script 不断修改不同 replica 的文件
- replica 的同步请求由 centra 来统一发送，便于测试

### Working Mode

- 所有的同步都以 `fetch_delta` 为基本模式
- `request_sync(A, B)` 可以表示为向`A`发送`fetch_delta(B)`的请求，也就是`B -> A`的同步
- `request_sync`可以由centra来发送，在没有centra的时候，也可以在replica之间独立完成

### Replica 端 RPC

- 接受来自中心服务器的同步请求`request_sync`
- 接受来自其他replica的同步`fetch_delta`

并发带来的同步问题

- 需要结合tra的具体算法来考虑
- 不同的文件之间同步没有冲突，直接并发执行即可
- 当tra中心服务器向同一个replica发送同一个路径的多次同步请求的时候，需要注意顺序问题？还是不会有时间上的overlap？
- 一个replaic向另外一个replica fetch delta的时候，需要考虑fetch到的delta是否是它想要的版本的delta？还是说这个问题由tra来保证？

资源抢占问题

- `request_sync`的请求并发执行，必定会有data race的问题
- 本身server接受`request_sync`的顺序就不能保证
- 文件抽象内的data race：不懂，看具体代码实现，应该直接加锁就好
- 文件同时写如何的data race：需要使用锁来保证
  - 全局资源，对于Hashmap加上RwLock, 对于文件（路径字符串）加上Mutex
  - 或者使用channel发送给主线程，主线程一个一个处理，同一个文件的请求需要保证先后顺序（可能不需要锁）

### Directories Watcher

- 需要监听每一个文件夹的变化
### Timestamp

- 利用 `inotify` 来定义一次文件修改/创建/删除的 atomic 操作
- 对于每一个 atomic 操作， local 的 timestamp 都会 ++

Or 

- ~~本地的修改全部算成一次，只有同步的时候会让 local time ++~~

Freeze


- 在确认了一次 sync 的请求之后，应该将两端的 file 全部 freeze ，然后进行 sync

### Tools

- 文件夹遍历：`walkdir`
- Rsync 算法：`fast_rsync`
- 异步框架：`tokio`, `futures`
- gRPC：`tonic`, `prost`, `tonic-build`
- 正则匹配：`regex`
- 文件监测：`inotify`