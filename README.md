# WeCom Local CLI

[![CI](https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml/badge.svg)](https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![Runtime](https://img.shields.io/badge/runtime-read--only-green)
![Status](https://img.shields.io/badge/status-experimental-orange)
![License](https://img.shields.io/badge/license-Apache--2.0-blue)

面向 AI Agent 的企业微信 Desktop 本地只读查询 CLI。

`wecom-local` 让 Codex、Claude、Hermes 等 Agent 可以用稳定 JSON 查询本机
企业微信 Desktop 中当前账号已经可见的数据：会话、消息、群成员、关键词搜索和
基础统计。它不上传数据、不发送消息、不连接官方 WeCom API，也不扩大账号可见范围。

[English README](README.en.md)

## 解决什么问题

很多企业微信信息只存在于 Desktop 客户端的本地可见状态里。官方 API 适合企业
Bot、审批、通讯录和系统集成，但它通常无法覆盖一个普通用户在桌面端已经能看到
的所有历史会话。

AI Agent 要分析这些信息时，常见低质量路径是截图、复制粘贴、手工导出文件，或
让 Agent 猜会话名。这样会带来几个问题：

- 输出不可复现：同一个任务下次很难取到同样结构的数据。
- 隐私边界模糊：正文、成员、真实会话 id 容易被误贴到日志或仓库。
- Agent 难以定位：会话名相似时容易读错群。
- 分析不完整：只看消息正文时，缺少群成员规模、活跃发言者和扫描窗口信息。

`wecom-local` 的目标是把这些本地可见数据变成可解释、可恢复、只读、结构化的
Local Query。

## 适合场景

- 让 Codex/Claude/Hermes 先查会话，再读取最近消息并总结行动项。
- 分析某个项目群的近期讨论窗口，同时区分群成员总数和实际发言人数。
- 在不写导出文件的前提下，对一个会话做关键词搜索或统计。
- 为个人归档生成一次性 JSON/Markdown 输出，随后由本地 Agent 处理。
- 发布前检查本机 WeCom 数据库形态，判断未来 Local Store Reader 是否值得继续
  研究。

## 当前状态

实验性 macOS 原型。核心能力已经能在本机运行，但公开发布前仍需要更多 WeCom
Desktop 版本兼容证据。

| 能力 | 命令 | 状态 |
| --- | --- | --- |
| 运行环境检查 | `wecom-local doctor --json` | 已实现 |
| 会话发现 | `wecom-local conversations [--query <text>]` | 已实现 |
| 会话消息 | `wecom-local history <conversation-reference>` | 已实现 |
| 群成员 | `wecom-local members <conversation-reference>` | 已实现 |
| 会话内搜索 | `wecom-local search <query> --in <conversation-reference>` | 已实现 |
| 会话统计 | `wecom-local stats <conversation-reference>` | 已实现 |
| 成员参与统计 | `wecom-local stats <conversation-reference> --include-members` | 已实现 |
| 可选导出 | `wecom-local export <conversation-reference>` | 已实现 |
| 本地存储探测 | `wecom-local store-probe --json` | 已实现 |
| 直接数据库读取 | Local Store Reader | 未实现，仍在安全验证阶段 |
| 通讯录查询 | `contacts` | 未实现 |

## 安装

先从源码构建：

```bash
cargo build --release
```

构建后的二进制文件位于：

```bash
target/release/wecom-local
```

可以按需加入 PATH：

```bash
ln -sf "$PWD/target/release/wecom-local" "$HOME/.local/bin/wecom-local"
```

## 快速开始

检查运行环境：

```bash
wecom-local doctor --json
```

探测本地 WeCom 数据库形态，不读取消息、联系人或成员行值：

```bash
wecom-local store-probe --json
```

列出本地可见会话：

```bash
sudo wecom-local conversations
sudo wecom-local conversations --query "Example"
```

读取一个会话：

```bash
sudo wecom-local history "Example Group" -n 100 --format json
```

列出群成员：

```bash
sudo wecom-local members "Example Group" --format json
```

搜索一个会话：

```bash
sudo wecom-local search "roadmap" --in "Example Group" -n 20 --json
```

统计一个会话：

```bash
sudo wecom-local stats "Example Group" --max-scan 1000 --include-members --json
```

写入一次性本地导出文件：

```bash
sudo wecom-local export "Example Group" \
  --format markdown \
  --output ./.local/wecom-local/example-conversation.md
```

`conversation-reference` 可以是 `conversations` 返回的会话 id，也可以是唯一匹配
的 display-name 查询。display-name 匹配到多个会话时会失败，不会自动选择“最像”
的会话。

## Agent 工作流

推荐让 Agent 按这个顺序运行：

```text
doctor -> conversations --query -> history -> members -> stats/search -> 分析
```

示例指令：

```text
请用 wecom-local 查询 "Example Group" 最近 200 条消息，列出群成员字段，
再给我一个只包含行动项、未决问题和活跃发言人数的摘要。不要写导出文件。
```

Agent 集成应调用二进制并解析 JSON 输出，不应在 Skill、插件或提示词里重新实现
Runtime Bridge。

## 输出示例

会话发现：

```json
{
  "query": null,
  "total_count": 2,
  "matched_count": 2,
  "conversations": [
    {
      "conversation_id": "R:0000000000",
      "conversation_name": "Example Group",
      "conversation_type": 2,
      "last_message_id": 123,
      "modify_time_text": "2026-05-18 01:00:00"
    }
  ]
}
```

会话消息：

```json
{
  "conversation_id": "R:0000000000",
  "conversation_name": "Example Group",
  "total_message_ids": 128,
  "exported_count": 50,
  "messages": [
    {
      "message_id": 1,
      "sender_display": "Alice",
      "send_time_text": "2026-05-18 01:00:00",
      "text": "hello",
      "display_text": "hello"
    }
  ]
}
```

完整 JSON 结构见 [docs/schema.md](docs/schema.md)。

## 安全与隐私边界

- 只读取当前登录的 macOS WeCom Desktop 账号已经本地可见的数据。
- Runtime Bridge 保持只读；不会发送消息、修改会话或写回企业微信。
- 运行时命令通常需要 `sudo`，因为 macOS 进程附加权限由本机 PAM 管理。
- CLI 不保存 macOS 密码、不创建 askpass 脚本、不安装特权 helper。
- 公共文档、测试和示例只使用 synthetic data。
- `store-probe` 只读取数据库文件头和 plain SQLite schema 计数；它不会读取行值、
  输出密钥或写解密数据库。
- 真实聊天记录、真实会话 id、群名、联系人名、截图和导出文件不应提交到仓库。

权限细节见 [docs/macos-permissions.md](docs/macos-permissions.md)，安全边界见
[docs/safety.md](docs/safety.md)。

## 项目结构

```text
src/                  Rust CLI implementation
docs/                 schema, safety, permissions, release readiness
docs/adr/             accepted architecture decisions
skills/               thin Agent Skill instructions
opencli/              OpenCLI external CLI guidance
CONTEXT.md            domain glossary
```

## 开发与验证

改动 Rust 代码后运行：

```bash
cargo fmt
cargo test
cargo build
```

发布前建议额外运行：

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build --release
git diff --check
cargo package --list
```

贡献规则见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 文档索引

- [docs/schema.md](docs/schema.md): JSON 输出结构。
- [docs/safety.md](docs/safety.md): 安全边界和隐私红线。
- [docs/macos-permissions.md](docs/macos-permissions.md): macOS 授权、`sudo` 与 Touch ID。
- [docs/compatibility.md](docs/compatibility.md): 已验证环境和兼容性风险。
- [docs/release-readiness.md](docs/release-readiness.md): 开源发布前检查表。
- [CONTEXT.md](CONTEXT.md): 领域词汇。

## 路线图

- 补更多 WeCom Desktop 版本兼容证据。
- 降低重复 Runtime attach 的失败率，评估 batch query 或 attach 复用。
- 在 no-output proof 中研究 macOS WeCom wxSQLite3 key/page 验证路径。
- 设计 `contacts`，前提是 Runtime selector 足够清晰，且不会变成通讯录扫描器。
- 发布 Homebrew 或签名二进制。

## 相关项目

- [wx-cli](https://github.com/jackwener/wx-cli): 本地优先的微信数据 CLI。
- [OpenCLI](https://github.com/jackwener/OpenCLI): AI-native CLI hub。
- [WecomTeam/wecom-cli](https://github.com/WecomTeam/wecom-cli): 企业微信官方 API/Bot CLI。

`wecom-local` 的定位是企业微信 Desktop 本地只读查询能力，和官方 API/Bot 自动化
属于不同方向。

## License

Apache-2.0. See [LICENSE](LICENSE).
