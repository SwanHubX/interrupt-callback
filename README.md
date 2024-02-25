## Interrupt-Callback

由于腾讯云竞价实例在被释放时无法设置回调通知，只能主动查询释放状态，参见：[SwanHubX/awesome-ops#23](https://github.com/SwanHubX/awesome-ops/issues/23)。

故使用Rust编写一个简单的应用程序，功能是在服务器后台运行，循环检查当前的释放状态，一旦即将被释放（通常提前2分钟）即发送一个回调请求。