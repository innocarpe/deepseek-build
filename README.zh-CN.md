<div align="center">

[English](README.md) · **简体中文** · [日本語](README.ja-JP.md) · [한국어](README.ko-KR.md)

<!-- 临时 hero 图片来源：deepseek-ai/DeepSeek-V2 figures/logo.svg（DeepSeek-V3 沿用）。 -->
<a href="https://github.com/deepseek-ai/DeepSeek-V3">
  <img src="assets/deepseek-logo.svg" width="60%" alt="DeepSeek logo">
</a>

<h1>DeepSeek Build</h1>

<p><strong>DeepSeek 原生编码 · Grok 级执行。</strong></p>

<p>
  一款全屏终端编码代理（coding agent），围绕 DeepSeek 模型提供安全编辑、
  缓存感知会话与并行执行能力。
</p>

<p>
  <a href="https://github.com/innocarpe/deepseek-build/releases"><img alt="GitHub release" src="https://img.shields.io/github/v/release/innocarpe/deepseek-build?style=flat-square&label=release"></a>
  <a href="https://www.npmjs.com/package/@innocarpe/deepseek-build"><img alt="npm version" src="https://img.shields.io/npm/v/%40innocarpe%2Fdeepseek-build?style=flat-square&label=npm"></a>
  <a href="LICENSE"><img alt="Apache 2.0 license" src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square"></a>
</p>

<p>
  <a href="#快速开始">快速开始</a> ·
  <a href="#为什么选择-deepseek-build">为什么选择 DeepSeek Build</a> ·
  <a href="#工作原理">工作原理</a> ·
  <a href="#文档">文档</a> ·
  <a href="#参与贡献">参与贡献</a>
</p>

</div>

<p align="center">
  <img src="assets/deepseek-build-welcome.png" alt="DeepSeek Build 欢迎界面——由 dsb 打开的全屏 DeepSeek 代理 TUI" width="85%">
</p>

> [!NOTE]
> **产品状态：** `5.x` 系列是 owner-bar 完整产品线。**`5.5.0`** 是 vision-complete
> 冻结版本——Path A 上的 Deep Code（L1）、Reasonix（L2）与 Grok 吞吐能力（L3）
> 已全部完成。[`5.0.0` 切割](docs/product/evidence/CUT_5_0_0_2026-08-07.md) 通过了
> Path A 账本与独立评审。**npm 与 GitHub Latest 已发布 `5.5.0`。**
> 更早的 `3.x` 与 `4.x` 标签在[版本历史](docs/product/versions/README.md)中记录为部分尝试。

## 快速开始

从 npm 安装、添加你的 DeepSeek API 密钥，然后打开 TUI：

```bash
npm install -g @innocarpe/deepseek-build
deepseek-build setup
deepseek-build
```

注册表安装需要 Node.js 18 或更高版本，并在存在匹配的发布资源时使用预编译
二进制。该路径不需要 Rust；平台与源码回退细节见
[npm 安装指南](docs/user-guide/05-npm.md)。

`deepseek-build` 是主命令。`dsb` 是完全受支持的短别名，行为相同且使用完整
语义化版本号：

```bash
deepseek-build --version
dsb --version
```

如果安装程序报告产品 bin 目录不在 `PATH` 中，请先将其加入：

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
```

## 为什么选择 DeepSeek Build

| 能力 | 含义 |
| --- | --- |
| **DeepSeek 原生** | DeepSeek API 默认、Flash/Pro 路由、推理强度（reasoning effort），以及 DeepSeek 品牌 TUI。 |
| **安全编辑** | 基于版本的片段编辑（snippet）与失败即关闭的工作区权限，而非静默整文件替换。 |
| **长会话经济性** | 稳定的提示词前缀、惰性技能加载与工具调用修复，让续接的会话保持连贯且缓存友好。 |
| **吞吐能力** | 并行工具、后台 Shell 任务、子代理与可选的 worktree，运行在安全与缓存层之下。 |
| **持久会话** | 续接最近的全屏会话，或直接定位到已保存的会话。 |

结果是一个编码代理：既保留 Grok 衍生执行引擎的速度，又把 DeepSeek 特有的
成本、编辑与权限规则纳入产品路径之中。

## 日常使用

```bash
# 打开全屏 TUI
deepseek-build

# 续接最近的全屏会话
deepseek-build --resume

# 执行一轮非交互式对话
deepseek-build run "Explain the architecture of this repository."

# 使用受信任的本地编码配置
deepseek-build --dogfood
```

`--dogfood` 允许在当前工作区内写入，并在策略允许下执行 Shell。工作区之外的
写入与删除仍被拒绝。

短命令可将示例中的 `deepseek-build` 替换为 `dsb`。

## 认证与配置

交互式安装会把 API 密钥以 `0600` 权限存入 `~/.deepseek-build/credentials.json`：

```bash
deepseek-build setup
deepseek-build auth status
deepseek-build auth logout
```

在 CI 或其他非交互环境中设置 `DEEPSEEK_API_KEY`；环境变量优先于凭据文件。
产品配置、凭据、会话与用户技能默认存放在 `~/.deepseek-build/` 下。

## 从源码构建

源码安装面向贡献者与不受支持的发布平台，需要 Rust 1.94 或更高版本以及
`protoc` 或 DotSlash；首次代理构建可能需要几分钟。

```bash
git clone https://github.com/innocarpe/deepseek-build.git
cd deepseek-build
./scripts/install.sh

deepseek-build --version
dsb --version
```

Cargo 与自定义前缀选项见[安装指南](docs/user-guide/01-install.md)。

## 工作原理

```text
deepseek-build | dsb
        │
        ▼
product launcher ── auth · config · model routing
        │
        ▼
deepseek-build-agent ── full-screen TUI · tools · sessions
        │
        ▼
DeepSeek API
```

三个层次有明确的所有权。更高吞吐的机制不得绕过其下的编辑、权限或缓存契约。

| 层 | 来源 | 负责 |
| --- | --- | --- |
| **L1** | [Deep Code CLI](https://github.com/lessweb/deepcode-cli) | 片段安全编辑、技能即上下文、副作用权限。 |
| **L2** | [Reasonix](https://github.com/esengine/DeepSeek-Reasonix) | 稳定前缀经济性、Flash/Pro 行为、工具调用修复。 |
| **L3** | [Grok Build](https://github.com/xai-org/grok-build) | 基础运行时、TUI、并行工具、子代理、后台工作与 worktree。 |

规范性冲突规则见[harness 哲学](docs/architecture/HARNESS_PHILOSOPHY.md)，
完整系统图见[SYSTEM_ARCHITECTURE.md](docs/architecture/SYSTEM_ARCHITECTURE.md)。

## 文档

| 从这里开始 | 用途 |
| --- | --- |
| [用户指南](docs/user-guide/README.md) | 安装、配置、日常使用与完整功能索引。 |
| [首次运行设置](docs/user-guide/00-setup.md) | API 密钥、凭据优先级与无头设置。 |
| [会话](docs/user-guide/03-sessions.md) | 全屏续接与行模式会话存储。 |
| [权限](docs/user-guide/08-permissions.md) | 交互式询问、无头拒绝与工作区边界。 |
| [子代理](docs/user-guide/11-subagents.md) · [后台任务](docs/user-guide/12-background-tasks.md) · [Worktree](docs/user-guide/13-worktrees.md) | L3 执行表面。 |
| [已知限制](docs/product/KNOWN_LIMITS.md) | 当前打包、实时冒烟与平台边界。 |
| [产品 SSOT](docs/product/SSOT.md) | 产品文档冲突时由谁裁决。 |

## 开发

```bash
cargo build -p dsb-cli
cargo test --workspace
./scripts/check-semver.sh
./scripts/test-owner-bar.sh
```

根 Rust 工作区覆盖产品 crates。日常检查请避免对 vendor 全量运行 Cargo；
owner-bar 脚本使用有界的产品路径。

crate 映射见 [crates/README.md](crates/README.md)。仓库映射与文档所有权见
[docs/README.md](docs/README.md)。

## 参与贡献

开始前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。所有有意义的工作都通过
聚焦的 PR 落地：原子式 Conventional Commit、已有 kind 标签、诚实的测试证据，
以及 [PR 撰写指南](docs/contributing/pr-body-standard.md) 规定的评审叙事。

## 许可证

DeepSeek Build 使用 [Apache License 2.0](LICENSE)。内置与第三方代码保留其
原始许可；见 [NOTICE](NOTICE)。
