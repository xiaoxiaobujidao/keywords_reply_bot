# Keywords Reply Bot

一个用 Rust 编写的 Telegram 群组关键词自动回复机器人。当群成员发送的消息包含已配置的关键词时，机器人会自动回复对应内容。

## 功能

- **关键词匹配**：消息文本包含关键词即触发回复（子串匹配）
- **群组隔离**：每个群组独立维护关键词列表
- **管理员权限**：添加、删除关键词仅群组管理员可用
- **自动清理**：机器人发送的回复消息在 40 秒后自动删除
- **HTML 格式**：支持 HTML 格式的回复内容，添加命令时可用 `` ` `` 包裹代码片段

## 命令

| 命令 | 说明 | 权限 |
|------|------|------|
| `/add <关键词> <回复内容>` | 添加或更新关键词回复 | 管理员 |
| `/del <关键词>` | 删除指定关键词 | 管理员 |
| `/del_all` | 删除当前群组所有关键词 | 管理员 |
| `/all` | 查看当前群组所有关键词 | 所有人 |
| `/help` | 显示帮助信息 | 所有人 |

### 使用示例

```
/add 你好 欢迎加入本群！
/add 规则 请遵守群规，禁止广告。
/del 你好
/all
```

## 环境要求

- [Rust](https://www.rustup.rs/) 1.85+（项目使用 Rust 2024 edition）
- 一个 Telegram Bot Token（通过 [@BotFather](https://t.me/BotFather) 创建）

## 快速开始

### 1. 克隆并编译

```sh
git clone <repo-url>
cd keywords_reply_bot
cargo build --release
```

### 2. 配置 Bot Token

首次运行时，程序会提示输入 Token，也可提前创建 `bot_token` 文件：

```sh
echo "YOUR_BOT_TOKEN" > bot_token
```

> `bot_token` 已加入 `.gitignore`，请勿提交到版本库。

### 3. 运行

```sh
cargo run --release
```

启动后程序会：

1. 读取配置并验证 Bot Token
2. 连接 SQLite 数据库并自动执行迁移
3. 开始轮询 Telegram 消息

### 4. 将机器人加入群组

1. 在 Telegram 中将机器人添加到目标群组
2. 确保机器人有发送消息的权限
3. 由管理员在群内使用 `/add` 命令配置关键词

## 配置

| 配置项 | 说明 | 默认值 |
|--------|------|--------|
| `bot_token` 文件 | Telegram Bot Token | 首次运行时交互输入 |
| `DATABASE_URL` 环境变量 | 数据库连接字符串 | `sqlite:keywords_reply_bot.db` |

示例：

```sh
export DATABASE_URL="sqlite:keywords_reply_bot.db"
```

## 项目结构

```
keywords_reply_bot/
├── src/
│   ├── main.rs        # 入口
│   ├── config.rs      # 配置加载
│   ├── bot.rs         # Telegram 轮询与消息分发
│   ├── handlers.rs    # 命令处理与关键词匹配
│   ├── database.rs    # 数据库连接与迁移
│   └── entities.rs    # 数据模型
├── migration/         # SeaORM 数据库迁移
├── docker-compose.yaml
└── release.sh         # 交叉编译与部署脚本
```

## 数据库

使用 SQLite 存储关键词，表结构如下：

| 字段 | 类型 | 说明 |
|------|------|------|
| `group_id` | BIGINT | Telegram 群组 ID（联合主键） |
| `keywords` | TEXT | 关键词（联合主键） |
| `reply` | TEXT | 回复内容 |

迁移在程序启动时自动执行。如需手动管理迁移，参见 `migration/README.md`。

## 部署

### 交叉编译（Linux musl 静态链接）

项目已配置 `x86_64-unknown-linux-musl` 目标，需安装 musl 工具链：

```sh
# macOS
brew install musl-cross

# 添加编译目标
rustup target add x86_64-unknown-linux-musl

# 编译
cargo build --target x86_64-unknown-linux-musl --release
```

编译产物位于 `target/x86_64-unknown-linux-musl/release/keywords_reply_bot`。

### Docker Compose

```sh
# 先完成 musl 编译，将二进制放在项目根目录
cp target/x86_64-unknown-linux-musl/release/keywords_reply_bot ./

docker compose up -d
```

`docker-compose.yaml` 使用 Debian 基础镜像，挂载项目目录并运行 `./keywords_reply_bot`，配置了资源限制、日志轮转和自动重启。

### 远程部署

`release.sh` 提供了一键编译并同步到远程服务器的示例：

```sh
./release.sh
```

请根据实际环境修改其中的 SSH 主机与路径。

## 技术栈

- [frankenstein](https://crates.io/crates/frankenstein) — Telegram Bot API 客户端
- [SeaORM](https://www.sea-orm.org/) — 异步 ORM
- [Tokio](https://tokio.rs/) — 异步运行时

## 许可证

未指定开源许可证，使用前请与项目维护者确认。
