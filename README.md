<h1 align="center">WeCom Local CLI</h1>

<p align="center">
  <a href="https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/BobbyCats/wecom-local/actions/workflows/ci.yml/badge.svg"></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-macOS-lightgrey">
  <img alt="Language" src="https://img.shields.io/badge/language-Rust-orange">
  <img alt="Runtime" src="https://img.shields.io/badge/runtime-read--only-green">
  <img alt="Status" src="https://img.shields.io/badge/status-experimental-orange">
  <img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue">
</p>

<p align="center">
  <a href="README.en.md">English README</a>
</p>

让 Agent 看懂本机企业微信里的工作对话。

企业微信里最麻烦的不是消息太少，而是重点太散：进展、解释、责任、截止时间和真
正卡点混在一起。N 条消息看完，还是不知道谁负责、什么时候交、问题到底卡在哪。

`wecom-local` 是一层本地只读查询能力：从当前 macOS WeCom Desktop 已登录账号
可见的数据里读取会话、消息、群成员和统计，交给 Agent 继续分析。它不上传数据、
不发送消息、不连接官方 WeCom API，也不扩大账号可见范围。

JSON 只是机器接口。核心不是导出聊天记录，而是给 Agent 一条稳定、可恢复、失败
可解释的本地查询路径。

## 为什么做这个

微信已经有类似 `wx-cli` 的本地查询工具，可以把聊天记录交给本地脚本或 Agent 做
整理、搜索、复盘。企业微信更常出现在工作里，但一直缺少一条类似的本地读取路径。

我想解决的是这种很具体的问题：

- 员工回了一大段话，但我看不出他到底是在说明进展，还是在解释为什么没做成。
- 群里讨论很热闹，但没人把“谁做、做什么、什么时候给”说清楚。
- 一个问题反复出现，每次都被新的解释盖过去，最后没有人真的收尾。
- 老板以为自己已经讲清楚了，员工可能根本没理解重点。
- 事情出了问题以后，聊天记录里会有大量铺垫和解释，需要有人帮我把重点拎出来。

现在常见的办法都不太好：

- 截图：看得到，但 Agent 很难稳定复查。
- 复制粘贴：一次可以，长期不行，也容易漏上下文。
- 手工导出：会留下文件，隐私和清理都麻烦。
- 直接让 Agent 猜会话名：群名相似时容易读错。
- 官方 API：适合企业系统集成，不等于能读取桌面端当前用户已经看到的历史会话。

`wecom-local` 的目标不是做“监控”，也不是做官方 API 客户端。它更像一个本地读取
层：把本机可见的企业微信会话变成可解释、可恢复、只读、结构化的 Local Query。

真正的分析应该放在上层 Skill 里。比如让 Agent 把一段混乱的群聊翻译成：“重点是
什么、谁在负责、谁没有回答问题、下一步该问什么”。

## 适合场景

- 项目复盘：最近 N 条消息里到底定了什么、谁负责、哪些事还悬着。
- 跟进检查：之前交代过的事项有没有后续，哪些问题被绕过去了。
- 群活跃度：群里有多少成员，最近实际说话的人有多少。
- 沟通翻译：把一大段解释、铺垫、甩锅、跑题，压成几句真正有用的话。
- 风险提示：延期、没人负责、前后说法不一致、讨论很多但没有结论。
- 本地归档：只在需要时写一次 JSON/Markdown，其余时候直接让 Agent 走本地查询。

这些分析应该由上层 Skill 完成。CLI 只负责安全、只读、结构化地把本地数据取出来。

## 和官方 WeCom CLI 怎么配合

[WecomTeam/wecom-cli](https://github.com/WecomTeam/wecom-cli) 是企业微信开放平
台命令行工具，面向文档、智能表格、消息、通讯录、待办、会议、日程等官方 API 能
力。它适合“去做事”：创建文档、写智能表格、发消息、建待办、查日程、约会议。

`wecom-local` 走的是另一条路：从本机 WeCom Desktop 只读地拿到当前账号已经能看
到的会话、消息、群成员和统计。它适合“先看懂发生了什么”。

两者搭配起来，工作流会更完整：

```text
wecom-local 读取项目群最近 N 条消息
-> Agent 总结：重点、负责人、未回答问题、风险
-> WecomTeam/wecom-cli 创建待办 / 写入智能表格 / 生成会议议题 / 发一条确认消息
```

几个具体例子：

- 群里吵了半天没有结论：`wecom-local` 先把争议点和未确认事项提出来，再用官方
  CLI 建一个待办或会议议题。
- 员工回复很长但重点不清：`wecom-local` 让 Agent 提炼“他到底承诺了什么、没说
  什么”，再用官方 CLI 发一条确认口径。
- 项目群每天都有新消息：`wecom-local` 做本地摘要，官方 CLI 把摘要写入智能表格
  或文档。
- 老板想追进度：`wecom-local` 找出最近 N 条里还没收尾的任务，官方 CLI 创建跟进
  待办或安排会议。

简单说：`wecom-local` 负责读懂本机聊天现场，官方 `wecom-cli` 负责调用企业微信
开放能力把后续动作落下去。

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

## 为什么用 Rust

这个项目选择 Rust，主要是为了做一个轻、快、好分发的本地命令行工具：

- 单个二进制文件，用户不需要先装一堆运行时。
- 启动开销低，适合 Agent 一次次调用小命令。
- 类型和错误处理更适合维护稳定 JSON 输出。
- 本机文件、进程、权限这些事情，Rust 做起来比较直接。

实际查询速度不只取决于 Rust。企业微信 Runtime attach、macOS 权限、WeCom Desktop
当前状态都会影响耗时。所以这里不宣传具体“毫秒级”数字；更准确的说法是：CLI 本
身很轻，真正慢的地方通常在本机 Runtime 访问。

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
请用 wecom-local 查询 "Example Group" 最近 N 条消息，列出群成员字段，
再给我一个只包含行动项、未决问题和活跃发言人数的摘要。不要写导出文件。
```

更贴近日常工作的问法：

```text
请看一下 "Example Project" 最近 N 条消息：
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

```text
请把 "Example Group" 里最近 N 条关于发货延期的讨论翻译成普通话：
谁说了重点，谁只是在解释，真正的问题是什么，下一句我应该怎么问。
不要做道德评价，只看聊天记录里的事实。
```

Agent 集成应调用二进制并解析 JSON 输出，不应在 Skill、插件或提示词里重新实现
Runtime Bridge。

## 短名分析 Skills

命令行工具还是 `wecom-local`。Skill 调用名统一走短名：`wc-local` 做基础查询，
其他 `wc-*` 负责具体分析。这样日常少打字，也不会把项目名和 binary 改乱。

| Skill | 用途 |
| --- | --- |
| `wc-local` | 查会话、消息、成员、搜索和统计；只管拿数据，不做判断 |
| `wc-brief` | 把一个群最近 N 条消息翻成：发生了什么、谁负责、还差什么、下一句怎么问 |
| `wc-scan` | 扫你指定的一批群，找活跃群、没收尾的事、没人回的问题 |
| `wc-audit` | 查“问了没人回、答了但没承诺、没 owner、没时间”的地方 |
| `wc-style` | 看某个人在这个群里的沟通习惯；只给观察，不下性格判决 |
| `wc-draft` | 根据上下文起草下一句；只起草，不自动发送 |

这些 Skill 都只是编排 `wecom-local` 的 JSON 命令。它们不会重新实现 Runtime
Bridge，也不会默认写导出文件。

示例：

```text
用 wc-brief 看 "Example Project" 最近 N 条消息。别写长报告，直接告诉我：
这事到底卡在哪、谁该回复、下一句怎么问。
```

```text
用 wc-audit 查 "Example Team" 最近 N 条消息，找出问了没人回、答了但没承诺、
没负责人、没截止时间的地方。每条给一句可以直接发出去的追问。
```

```text
用 wc-style 看 "Example Group" 里 Alice 最近 N 条发言。只说能从聊天里看到的沟
通习惯，以及下次怎么问更容易拿到明确答案。不要给 MBTI 定型。
```

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

## 免责声明

`wecom-local` 是非官方开源项目，与腾讯、企业微信、WeCom 官方没有关联、授权、
认可或背书。`Tencent`、`WeCom`、`企业微信` 等名称只用于说明本工具的兼容目标；
相关商标、产品名称和标识归其权利人所有。

本工具只用于用户在自己的本机环境中，对当前登录的 WeCom Desktop 账号已经可见的
数据做只读查询、个人或内部分析。使用者需要自行确认使用方式符合适用法律法规、
公司制度、数据保护要求、劳动/隐私政策以及腾讯或企业微信相关服务条款。

不要把本工具用于未经授权的数据访问、绕过权限、监控他人、批量抓取、传播私人内
容，或任何违法、侵权、违反服务条款的用途。

本项目按 Apache-2.0 许可 “as is” 提供，不承诺兼容所有 WeCom Desktop 版本，也
不对因使用本工具造成的数据泄露、账号风险、合规责任或其他后果承担责任。如果你
认为本项目存在侵权、安全或合规问题，请通过 [SECURITY.md](SECURITY.md) 联系维护
者。

## 项目结构

```text
src/                  Rust CLI implementation
docs/                 schema, safety, permissions, release readiness
docs/adr/             accepted architecture decisions
skills/               Agent integration and local analysis Skill instructions
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

## 参与贡献

欢迎提 issue 和 PR，尤其是：

- 新的 WeCom Desktop / macOS 版本兼容性证据。
- Runtime selector 失效、授权失败、重复 attach 失败这类可复现问题。
- 更好的 Agent Skill 用法、README 示例、安装体验。
- Local Store Reader 的安全 proof，但不要提交 key、memory dump、解密库或真实数据。

公开 issue / PR 里不要贴真实聊天内容、真实会话 id、群名、联系人名、截图或导出文
件。需要贴输出时，请先脱敏，只保留状态、数量、字段名和错误类型。完整规则见
[CONTRIBUTING.md](CONTRIBUTING.md)。

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
