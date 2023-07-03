#### 7.2

- 摸爬滚打地了解了一些基于 socket 的通信的知识
   - Rust TCPStream 的 shutdown ：可以生成EOF
   - socket 出现了 deadlock, [Example](https://stackoverflow.com/questions/44015638/simple-rust-tcp-server-and-client-do-not-receive-messages-and-never-terminates)
- 尝试构建了一个基于 socket 的简单通信

#### 7.3

- 用`inotify`写了一个简单的demo
- 一个简单的 file tree 的框架，用于不同 replica 的结构维护