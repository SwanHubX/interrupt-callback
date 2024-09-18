# Interrupt-Callback

各大云服务商提供的竞价实例在价格上非常有优势，非常适用于平时的开发、测试、验证等场景，但具有不定时释放的特点，容易导致环境丢失且需要手动恢复环境，比较麻烦。而相比于云服务器，本地服务器则更可能受到断网、停电、机器故障等因素影响导致本地服务掉线。

不管是云服务器还是本地服务器，我们都需要即时知道其出现的异常状态才能做出反应，甚至是完成一些自动化操作。那么该项目主要实现对竞价实例和本地服务器的状况监控，在检测到异常时触发回调。

## 功能

- 监控阿里云、腾讯云竞价实例的释放状态并触发警报
- 监控本地服务器的状态，如果失去连接则发送警报，通常是网络断连、突然断电等导致的情况
- 当前支持飞书 Webhook 消息

后面会添加更多功能，比如：集成更多的通知，监控系统资源占用情况，支持Webhook回调实现自动化操作等等

## 原理

### 监控竞价实例状态

阿里云和腾讯云的抢占式实例提供了接口用于查询实例释放状态，程序会每隔一段时间通过提供的接口查询状态。详见：

- [阿里云 ECS 抢占式实例](https://help.aliyun.com/zh/ecs/use-cases/query-the-interruption-events-of-preemptible-instances)
- [腾讯云 CVM 竞价实例](https://cloud.tencent.com/document/product/213/37970)

### 监控本地服务器

在断网、停电等突发情况发生时，服务器会瞬间丢失连接，因此我们需要一个服务端来监测客户端服务器的状态，通常可以选用更稳定的云服务器作为服务端，本地服务器则作为客户端与服务端连接。客户端会发送定时心跳给服务端告知其活跃状态，如果客户端断连，服务端会发出告警。

每台运行该程序的服务器既可作为服务端也可作为客户端。

## 使用

直接运行编译后的二进制文件，可配置项如下：

### 环境变量

- `CONFIG_PATH` 配置文件路径，不填则默认为空配置
- `SERVER_PORT` 服务端监听端口，默认为 `9080`

### 配置文件

该项目基于 [TOML](https://toml.io/en/) 配置文件配置运行需要参数，示例为：

```toml
name = "ikun101"
provider = "AliCloud"
interval = 10

[alert.feishu]
webhook = "https://open.feishu.cn/open-apis/bot/v2/hook/-"
secret = "-"

[keepalive]
period = 30

[keepalive.server]
key = "hello"
num = 2

[keepalive.client]
uri = "ic://default:hello@172.16.101.10:9080"
```

| 参数                 | 描述                                                         | 必填 | 默认          |
| -------------------- | ------------------------------------------------------------ | ---- | ------------- |
| name                 | 实例名称                                                     | 否   | “”            |
| provider             | 服务器类型，有：`AliCloud` - 阿里云实例，`TencentCloud` - 腾讯云实例，`LocalHost` - 本地服务器 | 否   | `LocalHost`   |
| interval             | 查询竞价实例状态的间隔，单位为秒                             | 否   | 10            |
| alert                | 集成的警报类型，当前支持飞书[自定义机器人](https://open.feishu.cn/document/client-docs/bot-v3/add-custom-bot) | 否   |               |
| alert.feishu.webhook | 飞书机器人webhook地址                                        | 是   |               |
| alert.feishu.secret  | 飞书机器人密钥                                               | 是   |               |
| keepalive            | 心跳检测，包括客户端和服务端。同一个运行实例支持同时开启客户端和服务端 | 否   |               |
| keepalive.period     | 心跳间隔，单位为秒                                           | 否   | 30            |
| keepalive.server.key | 服务端预设的密钥                                             | 否   | “”            |
| keepalive.server.num | 服务端看门狗支持每个客户端缺勤的次数，看门狗每 `keepalive.period` 秒巡查一次 | 否   | 4（大概2min） |
| keepalive.client.uri | 服务端连接串                                                 | 是   |               |

服务端连接串的格式为：

```
ic://default:{key}@{host}:{port}
```

- `ic://` 为固定字段，基于TCP协议，取自 `interrupt-callback` 首字母
- `default` 为默认用户名
- `key` 为配置的服务端密钥，可不填，默认为空
- `host` 为服务端地址
- `port` 为TCP服务端运行的端口，可不填，默认为 `9080`，取自 `interrupt-callback` 字母的数量

例如：

```
ic://default@172.16.101.10            // 密钥为空，默认为9080端口
ic://default:hello@172.16.101.10:7788 // 密钥为hello，端口为7788
```

## P.S.

### 1. 为什么使用Rust编写这么简单的小项目？

使用Rust开发该项目是为了动手掌握Rust的基础知识并上手实际的项目，即“实践是检验真理的唯一标准”。

参考学习的文档有：

- [Rust 语言圣经](https://course.rs/about-book.html)

- [Rust 程序设计语言](https://kaisery.github.io/trpl-zh-cn/)

- [通过例子学 Rust ](https://rustwiki.org/zh-CN/rust-by-example/)
