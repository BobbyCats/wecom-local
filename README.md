# WeCom Local CLI

[![CI](https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml/badge.svg)](https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![Runtime](https://img.shields.io/badge/runtime-read--only-green)
![Status](https://img.shields.io/badge/status-experimental-orange)
![License](https://img.shields.io/badge/license-Apache--2.0-blue)

把本机企业微信里已经能看到的会话，安全地交给 Agent 查询和分析。

很多工作问题都藏在企业微信里：事情有没有被接住、谁在跟进、哪个问题反复说不清、
群里到底哪些人在参与。原生客户端适合聊天，但不适合让 Codex、Claude、Hermes 这
类 Agent 稳定地读取、搜索和统计。

`wecom-local` 做的事情很窄：只读本机 WeCom Desktop 当前账号已经可见的数据，并
输出稳定 JSON。它不上传数据、不发送消息、不连接官方 WeCom API，也不扩大账号可
见范围。

[English README](README.en.md)

## 为什么做这个

微信已经有类似 `wx-cli` 的本地查询工具，可以把聊天记录交给本地脚本或 Agent
做整理、搜索、复盘。企业微信更常出现在日常工作里，但这条路一直缺一块。

一个很常见的场景是：你在企业微信里安排了事情，过几天发现结果不对，回头翻群聊
才发现中间有很多模糊、跳跃、没人确认的地方。人可以慢慢翻，但 Agent 不能靠截图
和复制粘贴长期工作。

现在常见的办法都不太好：

- 截图：看得到，但 Agent 很难稳定复查。
- 复制粘贴：一次可以，长期不行，也容易漏上下文。
- 手工导出：会留下文件，隐私和清理都麻烦。
- 直接让 Agent 猜会话名：群名相似时容易读错。
- 官方 API：适合企业系统集成，不等于能读取桌面端当前用户已经看到的历史会话。

`wecom-local` 的目标不是做“监控”，也不是做官方 API 客户端。它只是把本机可见
的企业微信会话变成可解释、可恢复、只读、结构化的 Local Query。

## 适合场景

- 项目复盘：最近 200 条里到底定了什么、谁负责、还有什么没闭环。
- 跟进检查：之前交代的事情有没有后续，哪些问题被反复绕开。
- 群活跃度：群成员有多少，真正参与讨论的人有多少。
- 沟通清晰度：任务描述、反馈、确认动作是否清楚。
- 风险提示：延期、没人接、前后说法不一致、讨论一直没有结论。
- 本地归档：只在需要时写一次 JSON/Markdown，其余时候直接让 Agent 查询 JSON。

这些分析应该由上层 Skill 完成。CLI 只负责安全、只读、结构化地把本地数据取出来。

## 当前状态

实验性 macOS 工具，已经公开发布 v0.1.1。核心能力可以在本机运行，但仍需要更多
WeCom Desktop 版本兼容证据。

| 能力 | 命令 | 状态 |
| --- | --- | --- |
| 运行授权检查 | `wecom-local auth status --json` | 已实现 |
| 运行授权预热 | `wecom-local auth prepare` | 已实现 |
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

Apple Silicon Mac 推荐从 GitHub Release 下载预编译二进制：

```bash
curl -L -o wecom-local.tar.gz \
  https://github.com/BobbyCats/wecom-local/releases/download/v0.1.1/wecom-local-v0.1.1-aarch64-apple-darwin.tar.gz
tar -xzf wecom-local.tar.gz
sudo install -m 755 wecom-local-v0.1.1-aarch64-apple-darwin/wecom-local /usr/local/bin/wecom-local
```

也可以从源码构建：

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
wecom-local auth status --json
wecom-local auth prepare
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
sudo wecom-local members "Example Group" --full --format json
```

默认成员输出只包含 basic **Member Detail Scope**，用于降低成员账号、邮箱、手机
号和外部 id 被误贴到日志或提示词的风险。只有在确实需要完整本地可见 profile 字
段时才使用 `--full`。

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
auth status -> auth prepare -> doctor -> conversations --query -> history -> members -> stats/search -> 分析
```

示例指令：

```text
请用 wecom-local 查询 "Example Group" 最近 200 条消息，列出群成员字段，
再给我一个只包含行动项、未决问题和活跃发言人数的摘要。不要写导出文件。
```

更贴近日常工作的问法：

```text
请看一下 "Example Project" 最近 300 条消息：
1. 哪些事情已经有明确负责人？
2. 哪些事情被提到了但没有后续？
3. 哪些成员在实际推动讨论？
4. 哪些问题需要我今天追一下？
只输出结论和引用到的消息时间，不要写导出文件。
```

```text
请分析 "Example Team" 最近一周的沟通：
哪些任务说清楚了，哪些任务只是在来回讨论？
如果有风险，只按“需要确认 / 需要负责人 / 需要截止时间”分类。
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
  `auth status` 可以无提示检查当前授权缓存，`auth prepare` 可以通过系统
  `sudo`/PAM 交互预热授权。
- CLI 不保存 macOS 密码、不创建 askpass 脚本、不安装特权 helper。
- 公共文档、测试和示例只使用 synthetic data。
- `store-probe` 只读取数据库文件头、page shape 字节和 plain SQLite schema 计数；
  它不会读取行值、输出密钥/内存字节或写解密数据库。
- `members` 默认只输出 basic 成员字段；`--full` 会输出更多本地可见 profile 字
  段，使用后不要把原始结果贴到公开位置。
- 真实聊天记录、真实会话 id、群名、联系人名、成员信息、截图和导出文件不应提
  交到仓库。

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
