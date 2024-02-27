# Interrupt-Callback

由于腾讯云竞价实例在被释放时无法设置回调通知，只能主动查询释放状态，参见：[SwanHubX/awesome-ops#23](https://github.com/SwanHubX/awesome-ops/issues/23)。

故使用Rust编写一个简单的应用程序，功能是在服务器后台运行，循环检查当前的释放状态，一旦即将被释放（通常提前2分钟）即发送一个回调请求。

## 启动

### 传参

| 参数 | 参数位置 | 说明 |
| ------- | ------- | ------- |
| webhook_url  | 1  | 回调接口地址，必传  |
| token  | 2  | 进行接口回调时的凭证，选传  |

### 启动命令

后台启动

```shell
# 先进入到可执行文件所在位置
# 最后面的 & 表示后台运行，必须加上
nohup ./interrupt-callback http://localhost:6092/api/interrupt token_here &
```
