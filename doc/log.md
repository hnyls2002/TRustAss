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