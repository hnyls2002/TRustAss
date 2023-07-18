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

###  增量同步算法

- Rust 的 `fast_rsync` 库
- 一次RPC
  - A : send signature
  - B : diff with the data, send the patch back
  - A : receive the patch, apply the patch

#### 需要考虑的问题

多个文件的同步
- 开用户态线程，主线程处理字节流，解析元数据分发给用户态线程
- 如何确定分发的目标用户态线程：连接确立后得到用户态线程编号，后续的数据中将需要处理的用户态线程编号放到元数据中
- 单个 socket 接口处理多个用户态线程
- 一个文件多次同步，一致性？版本冲突？用 timestamp 来标识版本？加锁？

文件修改协议的同时进行

- 两次 socket 连接，并且分配不同的协程
- 协程在建立的时候就应该知道自己所需要处理的协议
- 建立一个协议的分发器，从而建立不同种类的协程？

Rust `TCPStream` 的这个 socket 接口

- read 和 write 的阻塞问题
- 是否足够上层：字节流能实现完整的发送和接受逻辑
- `shutdown` 的含义以及作用

### 测试

- 中心服务器从外部读入测试点，向不同的replica发送文件的修改命令
- 由专门的协议来进行测试点的输入

### 文件监测

- rust `inotify`库

### 用户态线程（异步处理模式?）

- Rust 的 `async` 和 `await` 机制, 实现异步处理
- 消息的分发（保证控制流分发的连续性）：`async_channel`库
- 协议的上下文管理：在协程中做

Response but Asynchronization Communication Mode

- 所有的通讯必须要有回复（一拍一拍来）
- 但是接受消息都是采用`tokio`的异步`TcpStream`来
- 不会出现消息交叉的情况
- 不同的消息也不会share相同的receive buffer

Multi-Directories

- 每个directory同步的监听模式是“你来我往”的
- 每个directory的处理逻辑是异步的
- 每个replica需要接受信息，然后将信息发送给多个directories的处理逻辑 (`async_channel`)

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

### 其他

#### 文件名的正则检查和绝对路径

- rust `regex`库
