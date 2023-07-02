### Socket

使用 `socket` 模块实现 IPC 通信。

#### 两种不同的网络编程模型

`std::net` 同步阻塞的网络
`tokio::net` 异步非阻塞的网络

#### 如何持续地接受 client 的请求

- loop 并且不断 accept
- 多线程并发处理：每次有新的请求时创建一个新的线程来监听新的任务。`listener.incoming`是一个阻塞调用，可以等待下一个消息。