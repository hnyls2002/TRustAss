### Socket

使用 `socket` 模块实现 IPC 通信。

#### 两种不同的网络编程模型

`std::net` 同步阻塞的网络
`tokio::net` 异步非阻塞的网络

#### 如何持续地接受 client 的请求

- loop 并且不断 accept
- 多线程并发处理：每次有新的请求时创建一个新的线程来监听新的任务。`listener.incoming`是一个阻塞调用，可以等待下一个消息。

### rsync 增量传输算法

![rsync差异检测示意图](assets/5ece748f0af4c29a8d5e382d195324c834076.png)

- A 按照固定长度划分
- B 滑动窗口，不断地check
- 通过弱摘要和强摘要双重校验，来判断是否相同。弱摘要可以用于快速判断。

### `fast_rsync` in Rust

- 先确定签名的参数，block的大小和MD4 哈希的大小
- 用`Signature::calculate`计算Hash
- 转换成`IndexedSignature`，用于比较
- `diff`用来计算Delta
- `apply`用来打补丁