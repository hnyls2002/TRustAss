#### 7.2

- 摸爬滚打地了解了一些基于 socket 的通信的知识
   - Rust TCPStream 的 shutdown ：可以生成EOF
   - socket 出现了 deadlock, [Example](https://stackoverflow.com/questions/44015638/simple-rust-tcp-server-and-client-do-not-receive-messages-and-never-terminates)
- 尝试构建了一个基于 socket 的简单通信

#### 7.3

- 用`inotify`写了一个简单的demo
- 一个简单的 file tree 的框架，用于不同 replica 的结构维护

#### 7.4

- 啥也没干，大致想了一下一些其他的结构，比如要怎么模拟不同机器上面的文件。
- 学习了正则表达式
- 写了一个简单的文件路径的checker, 用于检查文件路径的合法性，以及是否是绝对路径
- 参观MSRA, 暑期学校的欢迎晚宴，认识了MIT的大佬

#### 7.5

- 补完整了`file_tree`的基本结构，将一个本地的文件给抽成一颗抽象的树形结构
- 支持了文件夹的`tree`命令

#### 7.6

- 参加人工智能大会，花了一幅丁真，但是不像
- 了解了一下rust的`fast_rsync`，基于`librsync`的一个库，可以用于增量同步。
- 写了一个小的 demo, 用于测试`fast_rsync`的功能。

#### 7.7

- 了解了什么是一个protocol
- 了解了`protobuf` 这个库，可以用于协议内容的编码和解码
- 尝试构利用`protobuf`构建一个简单的文件同步协议
  - 目前还没有考虑到可能会存在的一些问题

#### 7.9 

- 简单了解了一下Rust 的 async programming
   - `async` and `await` 上层语法糖
   - `futures`, `poll`, `wake` 等相关概念
   - `tokio`, `async-std` 两个异步运行时的库