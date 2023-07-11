### 基本框架

- `tra` : 中心化的服务器，用于启动不同线程上面的replica同步服务
- `trasrv` : 不同的副本的同步服务
- 文件同步的过程：不需要经过中心化的服务器，直接在不同的副本之间进行同步

### 不同的机器模拟

- 采用不同的线程
- 由于使用的是同一台物理机的文件系统，所以以线程id作为标识，来区分不同的replica

### 通信

- 采用socket来进行通信，通过字节流的方式进行通信
- 中心服务器只用于处理一些信息的转发，不对data进行传输
- data的传输直接通过不同的replica之间进行传输，每对传输的thread之间都会建立一个socket连接
- 所有replica所在线程的端口由TcpStream::connect自动分配。并且这些端口可以被tra中心服务器知道。

###  增量同步算法

- Rust 的 `fast_rsync` 库
- 传输签名和增量补丁的过程
- 上层协议的设计

### 文件同步协议

基于 socket 连接的上层文件同步协议。

Simple Version

- Header 定长，元信息
- Payload 经过protobuf序列化之后得到的数据

Header

- First 3 bytes
  - 魔数用于验证第一次连接
  - 本次数据的类型
- Next 2 bytes
  - 本次数据的长度，max 65535 bytes
  - 不包括 header 的长度，但是包括 EOF (`0x4`) 的长度

Payload

- 用 protobuf 来解析
- 最后一个字节为`0x4`，用于表示数据的结束


Control Flow

1. Machine A : 发送同步请求，包括同步的文件路径
2. Machine B : 确认请求，并且做相应的准备 (创建文件，File::open())
3. Machine A : 发送replica A的signature
4. Machine B ：返回 delta(B, A)
5. Machine A : apply delta(B, A) 打补丁

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


### 其他

#### 文件名的正则检查和绝对路径

- rust `regex`库
