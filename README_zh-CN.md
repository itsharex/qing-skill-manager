# Qing Skill Manager

[English](README.md) | [中文](README_zh-CN.md)

> 在一个桌面应用中管理你设备上所有的 AI 技能 —— 跨项目、跨 Agent IDE、统一掌控。

<p align="center">
  <img src="docs/screenshots/zh-CN/local.jpg" width="720" alt="Skill 库 — 三栏工作台" />
</p>

随着 AI 编程 Agent 越来越多，散落在你设备上的 Skill 也越来越多：有的在 Cursor 里，有的在 Claude Code 里，有的在 OpenCode 里，不同项目还用着不同版本。Qing Skill Manager 给你**一个统一的地方**，看清全局、管理版本、把对的技能推送到对的位置。

基于 **Tauri 2 + Vue 3 + Rust** 构建。开源。跨平台。

## 为什么需要它

- 你同时使用**多个 Agent IDE**（Claude Code、Cursor、Codex、OpenCode……），每个都有自己的 Skills 目录
- 你有**多个项目**，每个项目需要不同的技能集或不同的版本
- 你已经记不清哪个技能在哪、哪个版本是最新的、哪些已经过期
- 你需要一个**单一信息源**来管理这台设备上的所有 AI 技能

## 界面预览

| Skill 库 | 商店检索 |
|:---:|:---:|
| ![Skill 库](docs/screenshots/zh-CN/local.jpg) | ![商店检索](docs/screenshots/zh-CN/market.jpg) |

| IDE 浏览 | 项目管理 |
|:---:|:---:|
| ![IDE 浏览](docs/screenshots/zh-CN/ide.jpg) | ![项目管理](docs/screenshots/zh-CN/project.jpg) |

## 核心功能

### 多 IDE 技能管理

每个 Agent IDE 都把技能存在自己的目录里。Qing Skill Manager 统一读取它们，清晰展示每个 IDE 里安装了什么，并支持从一个界面安装、卸载或同步。原生支持 **Claude Code、Cursor、Codex、OpenCode、OpenClaw** —— 还可以几秒内注册任意自定义 IDE。

### 多版本技能追踪

每个技能在本地仓库中可以有**多个版本**。支持任意两个版本的逐行对比（文件差异、元数据变更、相似度评分）。设置默认版本，为不同场景创建命名**变体**（如"精简版" vs "详细版"），并为特定项目锁定指定版本。版本历史记录来源（市场下载、项目导入、手动添加）和创建时间。

### 带分类的技能库

三栏式 **Skill 库**是主要工作台：

- **左侧边栏** — 搜索，按平台筛选（哪个 IDE），按状态筛选（已托管 / 未托管 / 仅插件），按名称或版本数排序
- **中间详情面板** — 选中技能的完整视图：描述、路径、各 IDE 安装状态、项目部署情况、快捷操作
- **右侧版本轨** — 完整版本历史、默认版本标记、每版本的 IDE 和项目计数、重命名/删除/对比/设为默认

技能被分为**已托管**（在你的仓库中追踪）、**未托管**（在某个 IDE 中找到但不在仓库中）和**仅插件**。未托管的技能可以一键"收编"到中央仓库。

### 按项目部署技能

注册你的项目，配置每个项目使用哪些 IDE，然后部署指定版本的技能。应用会**自动检测项目目录中已有的技能**，并在版本不一致时标记冲突。冲突解决提供三个选择：**保留**项目版本、用仓库版本**覆盖**、或**共存**（重命名后两者都保留）。

### 市场发现

在一个界面中搜索 **Claude Plugins**、**SkillsLLM** 和 **SkillsMP** 的技能。下载后直接进入本地仓库，随时可安装到任何位置。版本更新会自动检测。

## 支持的 IDE

| IDE | 全局 Skills 路径 | 项目 Skills 路径 |
|-----|-----------------|-----------------|
| Claude Code | `~/.claude/skills` | `.claude/skills` |
| Codex | `~/.codex/skills` | `.codex/skills` |
| Cursor | `~/.cursor/skills` | `.cursor/skills` |
| OpenClaw | `~/.openclaw/skills` | `.openclaw/skills` |
| OpenCode | `~/.config/opencode/skills` | `.opencode/skills` |

**+ 任意自定义 IDE**（指定名称 + Skills 目录路径即可）。

## 工作原理

```
市场 / 本地文件夹
      ↓ 下载 / 导入
  中央仓库  (~/.qing-skill-manager/skills)
      ↓ 安装（复制）             ↓ 克隆（复制 + 版本锁定）
  全局 IDE 目录                项目级 IDE 目录
  （所有项目可用）              （仅限特定项目）
```

1. **收集** — 从市场下载，或从本地文件夹导入。所有技能统一进入中央仓库。
2. **整理** — 浏览技能库，管理版本，创建变体，分类筛选。
3. **分发** — 全局安装到 IDE，或将特定版本克隆到特定项目。
4. **维护** — 应用追踪每个技能安装在哪里。版本不一致时自动检测冲突并引导解决。

## 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) (LTS 版本)
- [Rust](https://rustup.rs/)
- [pnpm](https://pnpm.io/)
- macOS: Xcode Command Line Tools

### 安装与运行

```bash
git clone https://github.com/qing-claw/qing-skill-manager.git
cd qing-skill-manager/skills-manager
pnpm install
pnpm tauri dev
```

### 构建

```bash
pnpm tauri build
```

### macOS 安全提示

应用尚未配置 Apple 开发者签名。首次启动可能会提示"应用已损坏"或"来自身份不明的开发者"。执行以下命令即可放行：

```bash
xattr -dr com.apple.quarantine "/Applications/qing-skill-manager.app"
```

## 数据来源

| 来源 | 地址 |
|------|------|
| Claude Plugins | `https://claude-plugins.dev/api/skills` |
| SkillsLLM | `https://skillsllm.com/api/skills` |
| SkillsMP | `https://skillsmp.com/api/v1/skills/search`（可能需要配置 API Key） |

## 技术栈

- **桌面端**: Tauri 2（Rust 后端 + WebView 前端）
- **前端**: Vue 3 + TypeScript + Vite
- **多语言**: 中文 & English（vue-i18n）

## 致谢

Qing Skill Manager 基于 [skills-manager](https://github.com/Rito-w/skills-manager) 原始项目继续开发。感谢原作者与所有贡献者。

## 许可证

MIT
