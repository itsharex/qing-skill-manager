# Qing Skill Manager

[English](README.md) | [中文](README_zh-CN.md)

**一个面向多 IDE AI Skill 分发与版本管理的重新包装 fork。**

Qing Skill Manager 用于搜索、导入、版本化管理并分发 AI skills 到受支持的 IDE 与项目工作区。它提供统一本地仓库、基于 clone/copy 的项目分发方式、项目侧冲突检测，以及更清晰的受管版本匹配能力，适合希望更稳定管理 skill 资产的个人开发者与团队。

项目基于 **Tauri 2 + Vue 3 + Rust** 构建，核心工作流包括：

- 从多个 marketplace 检索 skills
- 统一存入本地受管仓库
- 安装到全局 IDE 目录
- 克隆到项目级 IDE 环境
- 跟踪导入版本并清晰处理项目冲突

## 致谢

Qing Skill Manager 基于原始项目 [skills-manager](https://github.com/Rito-w/skills-manager) 继续开发。感谢原作者与所有贡献者提供的基础能力与实现。 

> 截图正在根据 Qing Skill Manager 新品牌重新制作。
> 为避免与上游项目混淆，已移除继承自原项目的旧截图。

## ✨ 核心特性

- 🧭 **按布局草图适配的 Library 工作台**：新增三栏式技能库视图，在保留原项目视觉语言的前提下统一浏览技能、检查版本与查看安装上下文
- 🔍 **聚合市场检索**：基于公开 Registry，一站式搜索全网优质 Skills
- 📦 **统一本地仓库**：集中化管理下载内容 (`~/.qing-skill-manager/skills`)
- 🚀 **一键极速分发**：将统一的本地 Skills 快速安装至各个目标 IDE
- 🛠️ **多维管理界面**：支持基于 IDE 的细粒度浏览、无痕安全卸载机制
- ⚙️ **项目管理**：支持项目管理，将 skills 克隆到项目下，并配置项目使用的 IDE

## 🎯 原生支持的 IDE（按字母顺序）

- **Claude Code**: `.claude/skills`
- **Codex**: `.codex/skills`
- **Cursor**: `.cursor/skills`
- **OpenClaw**: `.openclaw/skills`
- **OpenCode**: `.config/opencode/skills`（项目级: `.opencode/skills`）

## 📖 使用指南

### 📥 获取与使用

- **普通用户**：请从源码构建，或在你自己的 fork 中发布版本。
- **开发者**：拉取源码在本地运行，或进行深度定制。

### 🍎 macOS 安全使用要求

由于目前暂时未配置 Apple 开发者商业证书，初次打开应用可能会遇到“已损坏，无法打开”或提示“未知的开发者”等系统拦截。作为开发者或极客用户，您可以在终端执行以下命令进行安全放行：

```bash
xattr -dr com.apple.quarantine "/Applications/qing-skill-manager.app"
```

### 🔍 1) 市场浏览 (Market)

- 基于配置好的服务源，聚合展示全网可用的优质 Skills。
- 点击下载将自动入库至本地，若本地仓库已存在较旧版本，将高亮显示“更新”按钮。

### 🗂️ 2) 本地仓库 (Local Skills)

- 集中俯瞰已下载到设备底层仓库的所有 Skills。
- 点击“安装”，即可在弹出的面板中勾选一个或多个原生 / 自定义的 IDE 进行批量安装。

### 🧱 3) Library 工作台

- Local 标签页现在承载一个按布局草图适配的 **Library 工作台**，整体由左侧技能栏、中间详情区、右侧版本轨组成。
- 左栏负责搜索、多选与切换 skill；中栏聚焦描述、路径、来源、安装状态、项目映射与克隆到项目入口；右栏展示版本列表、默认版本与版本对比入口。
- 这个视图现在已经成为版本映射、项目使用情况、克隆到项目等交互的主前端承载面。

### ⌨️ 4) IDE 纬度管理 (IDE Browse)

- 灵活切换工作环境视角（如 VSCode 或 Cursor），独立查看各自已安装的技能列表。
- 安全卸载模块：安全移除已安装技能目录，避免相互干扰。
- 找不到您的生产力工具？只需在右上角轻松创建你的“自定义 IDE”。

## 🏗 前端架构补充说明

- `src/App.vue` 仍然是应用编排中心，但 Local 标签页现在会挂载 `src/components/library/LibraryWorkspace.vue`。
- `src/components/library/` 目录承载新的 Library 域组件：
  - `LibrarySidebar.vue`
  - `LibraryDetailPanel.vue`
  - `LibraryVersionRail.vue`
  - `LibraryWorkspace.vue`
- `src/composables/useLibraryWorkspace.ts` 基于已有本地 skills、IDE 安装信息、项目快照与版本元数据，派生出 Library 工作台所需的视图模型。
- Market / IDE / Projects / Settings 既有流程保持不变；本轮改造重点在前端结构与交互表现，不要求修改 Rust 后端。

## 👨‍💻 安装与开发

### 环境依赖

- Node.js (建议 LTS)
- Rust (通过 rustup 安装)
- macOS: Xcode Command Line Tools

### 本地开发

```bash
pnpm install
pnpm tauri dev
```

### 测试说明

- 测试脚本文档：[`docs/testing-guide.md`](docs/testing-guide.md)
- 推荐的项目检查命令：

```bash
pnpm run check:project
```

- 针对本次 Library 工作台前端改造，建议至少执行：

```bash
pnpm run typecheck
pnpm run build
```

### 打包发布

```bash
pnpm tauri build
```

## 📡 远程数据来源

- **Claude Plugins**: `https://claude-plugins.dev/api/skills`
- **SkillsLLM**: `https://skillsllm.com/api/skills`
- **SkillsMP**: `https://skillsmp.com/api/v1/skills/search`（由于跨域限制可能需要提供 API Key 配置）
- 下载 ZIP 请求代理: `https://github-zip-api.val.run/zip?source=<repo>`

## 🛠 技术栈

- 桌面端核心框架：**Tauri 2**
- 前端视图层：**Vue 3** + **TypeScript** + **Vite**
- 底层文件与系统逻辑：**Rust** (命令侧)

## 📄 License

MIT
