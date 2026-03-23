# Skills Manager 开发文档

本文面向首次接手 `skills-manager` 的开发者，目标是回答四个问题：

1. 这个项目是干什么的
2. 项目架构是怎么组织的
3. 本地应该怎么启动和构建
4. 如果要改功能，应该从哪里下手

---

## 1. 项目概览

`skills-manager` 是一个基于 **Tauri 2 + Vue 3 + TypeScript + Vite** 的桌面应用，用来统一管理 AI 开发环境中的 skills。

它的核心能力包括：

- 聚合多个远程 marketplace 的 skill 搜索结果
- 将 skill 下载到本地统一仓库
- 通过符号链接把本地 skill 安装到目标 IDE / 项目目录
- 浏览本地 skills、IDE 中已挂载 skills、项目级 skill 配置
- 检查应用更新

从代码实现来看，当前项目的能力边界可以概括为：

- **前端（Vue）负责 UI、状态组织、交互流程**
- **后端（Tauri/Rust）负责文件系统、目录扫描、下载与本地系统能力**
- **前后端通过 `invoke()` 调用 Tauri command 连接**

---

## 2. 技术栈与运行方式

### 前端

- **Vue 3**
- **TypeScript**
- **Vite 6**
- **vue-i18n**

### 桌面运行时

- **Tauri 2**
- Tauri 插件：
  - `@tauri-apps/plugin-dialog`
  - `@tauri-apps/plugin-opener`
  - `@tauri-apps/plugin-process`
  - `@tauri-apps/plugin-updater`

### 后端

- **Rust**

### 包管理器

- 项目根目录存在 `pnpm-lock.yaml`，因此实际包管理器是 **pnpm**

---

## 3. 目录结构总览

项目主目录结构可以先这样理解：

```text
skills-manager/
├─ src/                 # Vue 前端源码
│  ├─ components/       # 界面组件：Panel / Modal / Overlay
│  ├─ composables/      # 业务状态与逻辑封装
│  ├─ locales/          # 国际化文案
│  ├─ assets/           # 全局样式与静态资源
│  ├─ App.vue           # 根组件，应用编排中心
│  ├─ main.ts           # Vue 应用入口
│  └─ i18n.ts           # 国际化初始化
├─ src-tauri/           # Tauri / Rust 后端
│  ├─ src/
│  │  ├─ commands/      # Tauri command 实现
│  │  ├─ lib.rs         # Tauri 应用构建与命令注册
│  │  └─ main.rs        # Rust 入口
│  ├─ tauri.conf.json   # Tauri 配置
│  └─ Cargo.toml        # Rust 依赖
├─ skills/              # 随应用分发的内置 skill
├─ public/              # 前端静态资源
├─ docs/                # 文档与截图
├─ package.json         # 前端脚本与依赖
├─ pnpm-lock.yaml       # pnpm 锁文件
├─ vite.config.ts       # Vite 配置
└─ tsconfig.json        # TypeScript 配置
```

---

## 4. 应用入口与启动路径

### 4.1 前端入口

文件：`src/main.ts`

职责很简单：

1. 创建 Vue 应用
2. 注入 i18n
3. 挂载根组件 `App.vue`

也就是说，**前端所有功能的真实入口是 `App.vue`**。

### 4.2 根组件

文件：`src/App.vue`

`App.vue` 是整个应用的编排中心，主要负责：

- 顶部 tab 切换：`local` / `market` / `ide` / `projects` / `settings`
- 主题与语言切换
- 应用启动时的初始化逻辑
- 汇总多个 composable 的状态与动作
- 渲染所有 panel 与 modal

它不是一个“纯展示层”组件，而是一个**页面级 orchestration component**。

如果你要快速理解应用怎么跑起来，第一步就是读：

1. `src/main.ts`
2. `src/App.vue`
3. `src/composables/useSkillsManager.ts`

### 4.3 Tauri 入口

Rust 侧入口分两层：

- `src-tauri/src/main.rs`
- `src-tauri/src/lib.rs`

其中：

- `main.rs` 很薄，只负责调用运行入口
- `lib.rs` 才是真正的 Tauri app 初始化位置

`lib.rs` 里做的事情包括：

- 注册 Tauri 插件
- 注册所有前端可调用的 `invoke_handler`
- 开启单实例能力（桌面环境）

所以如果你要看“前端到底能调哪些原生命令”，从 `src-tauri/src/lib.rs` 开始最直接。

---

## 5. 前端架构导览：按模块解释每一块是干什么的

前端整体采用的是：

- **根组件 + composables 状态管理 + 展示组件**

这不是 Pinia/Vuex 风格，而是以 composable 为中心的轻量架构。

### 5.1 `src/components/`：界面层

组件大致分为三类。

#### A. Panel：主标签页内容

- `LocalPanel.vue`
  - 展示本地仓库中的 skills
  - 支持安装、删除、打开目录、刷新、导入等操作

- `MarketPanel.vue`
  - 搜索远程市场中的 skills
  - 支持下载、更新、加载更多、保存 market 配置

- `IdePanel.vue`
  - 浏览各 IDE 目录中的技能
  - 支持按 IDE 筛选、接管技能、卸载技能、添加自定义 IDE

- `ProjectsPanel.vue`
  - 管理项目列表
  - 负责项目选择、项目配置、项目导入/导出 skill 入口

- `SettingsPanel.vue`
  - 应用设置与更新相关 UI 的承载位置

#### B. Modal：流程型交互弹窗

- `InstallModal.vue`
  - 选择安装目标（全局 IDE 或项目）

- `UninstallModal.vue`
  - 卸载确认

- `ProjectAddModal.vue`
  - 添加项目

- `ProjectConfigModal.vue`
  - 配置项目的 IDE targets

- `ConflictResolutionModal.vue`
  - 处理导入 skill 时的重名/冲突问题

- `ProjectSkillImportModal.vue`
  - 展示项目中扫描到的 skills，执行导入

- `ImportToProjectModal.vue`
  - 将本地 skills 挂载到项目对应 IDE 目录

#### C. 基础反馈与辅助组件

- `Toast.vue`：全局消息提示
- `LoadingOverlay.vue`：全局忙碌状态覆盖层
- `DownloadQueue.vue`：下载队列与状态展示
- `UpdateChecker.vue`：应用更新 UI
- `MarketSettingsModal.vue`：远程源配置

### 5.2 `src/composables/`：核心业务层

这是项目最重要的目录。真正的业务逻辑主要都在这里。

#### `useSkillsManager.ts`

这是当前项目的**主业务中枢**。

它负责：

- 市场搜索
- 搜索缓存
- 下载队列
- 下载 / 更新 skill
- 扫描本地 skills
- 扫描 IDE skills
- 安装到全局 IDE
- 安装到项目目录
- 卸载 / 删除 / 导入 / 接管技能
- 项目技能扫描与冲突解决
- 一部分全局 busy 状态控制

可以把它理解成“应用服务层 + 页面控制器”的结合体。

如果你只读一个业务文件，就先读它。

#### `useProjectConfig.ts`

负责项目配置管理：

- 加载项目列表
- 添加/删除项目
- 维护选中项目
- 保存项目的 IDE targets
- 保存自动检测到的 IDE 目录

当前实现基于 `localStorage` 持久化。

#### `useIdeConfig.ts`

负责 IDE 选项配置：

- 加载默认 IDE 与自定义 IDE
- 添加自定义 IDE
- 删除自定义 IDE
- 记录上一次安装时勾选的目标

需要注意：

- 当前 `constants.ts` 里的 `defaultIdeOptions` 只默认配置了 **OpenCode**
- 这说明**代码当前偏 OpenCode-first / MVP 状态**

虽然 README 列了很多 IDE，但你改功能时应优先相信代码中的当前实现，而不是只看 README。

#### `useMarketConfig.ts`

负责远程市场配置：

- API Key 存储
- 各 market 是否启用
- market 状态展示

同样基于 `localStorage`。

#### `useUpdateStore.ts`

负责自动更新流程：

- 获取应用名与版本
- 启动时静默检查更新
- 手动检查更新
- 下载更新
- 重启安装

这个模块直接依赖 Tauri updater / process 插件。

#### `useToast.ts`

负责全局 toast 状态。

它属于典型的全局轻状态模块，供多个流程复用。

### 5.3 `src/locales/` 与 `src/i18n.ts`：国际化层

- `src/i18n.ts` 初始化 `vue-i18n`
- 支持语言：`zh-CN`、`en-US`
- 当前默认语言和 fallback 都是 `zh-CN`

语言切换逻辑在 `App.vue`，并使用 `localStorage` 记住用户选择。

### 5.4 `src/assets/`：样式与资源

- `src/assets/app.css` 是全局样式入口之一
- `App.vue` 中也保留了大量全局与 scoped 样式

这说明样式目前是**集中在页面根组件 + 局部组件样式**的混合方式，而不是完整的 design system 或独立 theme module。

---

## 6. 后端架构导览：Tauri / Rust 每块做什么

### 6.1 `src-tauri/src/lib.rs`

作用：**组装 Tauri 应用**。

主要职责：

- 引入 command 模块
- 注册插件
- 注册前端可调用的命令
- 在桌面环境下启用单实例

当前注册的命令包括：

- `search_marketplaces`
- `download_marketplace_skill`
- `update_marketplace_skill`
- `link_local_skill`
- `scan_overview`
- `uninstall_skill`
- `import_local_skill`
- `delete_local_skills`
- `adopt_ide_skill`
- `scan_project_ide_dirs`
- `scan_project_opencode_skills`
- `resolve_skill_conflict`

这些命令基本覆盖了前端所有核心流程。

### 6.2 `src-tauri/src/commands/`

这是 Rust 侧真正的业务实现层。

按命名可以分为两类：

#### `market.rs`

负责远程市场相关能力：

- 搜索 marketplace
- 下载 marketplace skill
- 更新 marketplace skill

#### `skills.rs`

负责本地技能与目录系统相关能力：

- 扫描总览
- 安装链接
- 卸载技能
- 导入本地 skill
- 删除本地 skill
- 接管 IDE skill
- 扫描项目 IDE 目录
- 扫描项目内 OpenCode skills
- 处理冲突

### 6.3 前后端交互方式

典型数据流是：

```text
Vue 组件
  -> composable action
  -> invoke("tauri_command", payload)
  -> Rust command
  -> 文件系统 / 网络 / 本地系统操作
  -> 返回结果给前端
  -> 更新响应式状态
  -> UI 重渲染
```

例如：

- MarketPanel 点击下载
- `useSkillsManager.addToDownloadQueue()` 入队
- `processQueue()` 调用 `download_marketplace_skill`
- Rust 下载 skill 到本地仓库
- 前端提示成功并重新扫描本地 skills

---

## 7. 数据与状态流

### 7.1 启动阶段

应用启动时会发生几件关键事情：

1. `main.ts` 挂载 `App.vue`
2. `App.vue` 在 `onMounted` 中：
   - 加载 locale
   - 加载 theme
   - 执行启动时更新检查
   - 加载项目列表
3. `useSkillsManager()` 内部在 `onMounted` 中：
   - 刷新 IDE 选项
   - 加载 market 配置
   - 执行一次 marketplace 搜索
   - 扫描本地 / IDE skills

所以首次进入应用时，界面已经会自动做一轮初始化拉取和本地扫描。

### 7.2 持久化策略

项目当前前端侧主要使用 `localStorage` 保存用户配置。

已确认的 key 包括：

- `skillsManager.locale`
- `skillsManager.theme`
- `skillsManager.ideOptions`
- `skillsManager.lastInstallTargets`
- `skillsManager.marketConfigs`
- `market-enabled`
- `skillsManager.projects`

这意味着：

- 用户配置是轻量、浏览器态的
- 不涉及复杂状态管理库
- 调试时可以直接观察和清空 localStorage 验证行为

### 7.3 下载队列

`useSkillsManager.ts` 中实现了一个串行下载队列：

- 新任务进入 `downloadQueue`
- `processQueue()` 找 `pending` 任务逐个执行
- 成功后写入 `recentTaskStatus`
- 延迟删除队列项并触发 `scanLocalSkills()`

这部分是市场下载功能的关键机制。如果你要改下载体验、并发策略、重试逻辑，从这里下手。

---

## 8. 本地启动与开发

### 8.1 环境要求

根据 `README.md`，本地开发前需要：

- Node.js（建议 LTS）
- Rust（通过 rustup 安装）
- macOS 下需要 Xcode Command Line Tools

### 8.2 常用命令

`package.json` 中定义的脚本如下：

```json
{
  "dev": "vite",
  "build": "vue-tsc --noEmit && vite build",
  "preview": "vite preview",
  "tauri": "tauri",
  "release": "node \"scripts/release.js\""
}
```

结合 `README.md` 与 `src-tauri/tauri.conf.json`，推荐开发方式是：

```bash
pnpm install
pnpm tauri dev
```

### 8.3 为什么是 `pnpm tauri dev`

`src-tauri/tauri.conf.json` 里配置了：

- `beforeDevCommand: "pnpm dev"`
- `devUrl: "http://localhost:1420"`
- `beforeBuildCommand: "pnpm build"`
- `frontendDist: "../dist"`

这意味着：

- 运行 `pnpm tauri dev` 时，Tauri 会先启动 `pnpm dev`
- `pnpm dev` 实际就是 `vite`
- Vite 默认固定在 `1420` 端口，由 `vite.config.ts` 保证
- Tauri 再加载这个 dev server 作为桌面应用前端

### 8.4 相关配置文件

#### `vite.config.ts`

开发期关键点：

- 端口固定为 `1420`
- `strictPort: true`
- 忽略监听 `src-tauri/**`
- 如果存在 `TAURI_DEV_HOST`，会启用 HMR host 配置

#### `tsconfig.json`

类型系统是严格模式：

- `strict: true`
- `noUnusedLocals: true`
- `noUnusedParameters: true`
- `noFallthroughCasesInSwitch: true`

所以改动时要按严格 TypeScript 项目来处理，不能偷懒。

### 8.5 构建与发布

前端构建：

```bash
pnpm build
```

完整桌面构建：

```bash
pnpm tauri build
```

自定义发布脚本：

```bash
pnpm release
```

### 8.6 macOS 注意事项

README 明确写了一个首次安装注意事项：

```bash
xattr -dr com.apple.quarantine "/Applications/skills-manager-gui.app"
```

原因是应用尚未配置 Apple 商业签名，首次打开可能被 Gatekeeper 阻止。

---

## 9. 改功能该从哪里下手

下面按常见需求给出入口建议。

### 9.1 改 tab 页内容或页面结构

先看：

- `src/App.vue`
- 对应的 `src/components/*Panel.vue`

因为 tab 切换和 panel 装配都在 `App.vue`。

### 9.2 改市场搜索 / 下载 / 更新逻辑

前端先看：

- `src/components/MarketPanel.vue`
- `src/composables/useSkillsManager.ts`

后端再看：

- `src-tauri/src/commands/market.rs`

### 9.3 改本地技能扫描 / IDE 浏览 / 安装卸载逻辑

前端先看：

- `src/components/LocalPanel.vue`
- `src/components/IdePanel.vue`
- `src/composables/useSkillsManager.ts`
- `src/composables/useIdeConfig.ts`

后端再看：

- `src-tauri/src/commands/skills.rs`

### 9.4 改项目管理与项目级挂载逻辑

先看：

- `src/components/ProjectsPanel.vue`
- `src/components/ProjectAddModal.vue`
- `src/components/ProjectConfigModal.vue`
- `src/components/ProjectSkillImportModal.vue`
- `src/components/ImportToProjectModal.vue`
- `src/composables/useProjectConfig.ts`
- `src/composables/useSkillsManager.ts`

这里最关键的是理解两件事：

- 项目列表本身由 `useProjectConfig` 管
- 实际扫描、链接、冲突处理由 `useSkillsManager` 触发 Tauri command 完成

### 9.5 改更新检查逻辑

先看：

- `src/composables/useUpdateStore.ts`
- `src-tauri/tauri.conf.json`

因为 updater endpoint 与前端更新状态逻辑分别在这两处。

### 9.6 改国际化文案

先看：

- `src/locales/zh-CN.ts`
- `src/locales/en-US.ts`
- `src/i18n.ts`

新增文案时要保持两个 locale 文件结构同步。

### 9.7 改样式 / 主题

先看：

- `src/App.vue`
- `src/assets/app.css`
- 具体组件内的 `<style scoped>`

主题切换依赖 `data-theme` 属性和 CSS variables。

---

## 10. 核心功能详解

### 10.1 IDE 浏览页面详解

**文件**: `src/components/IdePanel.vue`

#### 页面显示内容

**1. IDE 筛选器**
- 顶部一排按钮，切换查看不同 IDE 中的 skills
- 默认只显示 `OpenCode`（在 `constants.ts` 中配置）
- 可添加自定义 IDE

**2. 自定义 IDE 添加区**
- 输入框：IDE 名称
- 输入框：IDE 目录路径（如 `.myide/skills`）
- "添加 IDE" 按钮
- 已添加的自定义 IDE 以标签形式显示，可删除

**3. Skill 列表**
每个 skill 卡片显示：

| 信息 | 说明 |
|------|------|
| **序号 + 名称** | 如 "1. my-skill" |
| **IDE 标签** | 显示属于哪个 IDE |
| **来源类型** | "链接"（symbolic link）或 "本地"（local copy） |
| **管理状态** | "未托管"（橙色边框）或已托管 |
| **完整路径** | skill 的绝对路径 |

**4. 操作按钮**
- **打开目录** - 在文件浏览器中打开 skill 位置
- **纳管** - 将未托管 skill 纳入管理（仅对未托管显示）
- **卸载** - 从 IDE 中移除该 skill

**5. 批量操作**
- **纳管选中** - 批量纳管未托管 skills
- **卸载选中** - 批量卸载选中的 skills

#### 关键概念

**托管 vs 未托管**
- **托管 (managed)**: 通过 Skills Manager 安装，实际存储在 `~/.skills-manager/skills/`，IDE 目录中是符号链接
- **未托管 (unmanaged)**: 用户手动复制到 IDE 目录，不在 Manager 控制下

**链接 vs 本地**
- **链接 (link)**: skill 是符号链接，实际存储在 manager 目录
- **本地 (local)**: skill 是实际目录，不是链接

**注意**: 当前代码默认只配置了 OpenCode，其他 IDE 需要手动添加或在 `constants.ts` 中扩展 `defaultIdeOptions`。

---

### 10.2 Skill 安装流程详解

**核心命令**: `link_local_skill`（Rust 后端）

#### 安装到全局 IDE

**流程**:
1. 在 **Local Skills** 页面选中 skills
2. 点击"批量安装到编辑器"
3. `InstallModal` 弹出，显示两列选择：
   - **左列（全局 IDE）**: 安装到 IDE 全局配置目录
   - **右列（项目）**: 安装到特定项目的 IDE 目录
4. 选择全局 IDE（如 OpenCode）
5. 点击"安装到 IDE"
6. 调用 `link_local_skill` 命令：

```rust
invoke("link_local_skill", {
  request: {
    skillPath: "~/.skills-manager/skills/my-skill",
    skillName: "my-skill",
    linkTargets: [
      { name: "OpenCode", path: "~/.config/opencode/skills" }
    ]
  }
})
```

**实际执行**（Rust 后端）:
1. **验证**: 确认 skill 路径在 `~/.skills-manager/skills/` 内
2. **创建符号链接**: 在 IDE 目录创建指向 manager 目录的 symlink
   - Unix: `ln -s ~/.skills-manager/skills/my-skill ~/.config/opencode/skills/my-skill`
   - Windows: `mklink /J` (junction)
3. **返回结果**: 哪些目标链接成功，哪些被跳过

#### 项目级 Skill 转成 IDE 全局

**场景**: 你已经在项目中手动安装了 skill，想把它转成 IDE 全局可用。

**方法 1: 纳管 + 重新安装**

1. **在 IDE 浏览页面找到项目中的 skill**
   - 如果 skill 显示为"未托管"（橙色边框）

2. **点击"纳管"按钮**
   - 执行 `adopt_ide_skill` 命令
   - 将 skill 从项目目录**复制**到 `~/.skills-manager/skills/`
   - 在项目目录**创建符号链接**（替换原目录）
   - 现在 skill 已是"托管"状态

3. **在 Local Skills 页面安装到全局 IDE**
   - 选中刚纳管的 skill
   - 点击"安装到编辑器"
   - 选择全局 IDE（如 OpenCode）
   - 执行 `link_local_skill`，在全局 IDE 目录创建符号链接

**方法 2: 从项目导入**

1. **在项目管理页面**
   - 选择项目
   - 点击"从项目导出" / "扫描项目中的 Skills"
   - 执行 `scan_project_opencode_skills` 命令扫描 `.opencode/skills` 目录

2. **导入到本地**
   - 选择要导入的 skills
   - 解决冲突（如果有）
   - 执行 `import_local_skill` 将 skill 复制到 `~/.skills-manager/skills/`

3. **安装到全局 IDE**
   - 回到 Local Skills 页面
   - 选中刚导入的 skills
   - 点击"安装到编辑器" → 选择全局 IDE

#### 安装目标对比

| 安装目标 | 存储位置 | 适用范围 | 命令 |
|---------|---------|---------|------|
| **全局 IDE** | `~/.config/opencode/skills/` (symlink) | 所有项目 | `link_local_skill` |
| **项目级** | `~/project/.config/opencode/skills/` (symlink) | 仅该项目 | `link_local_skill` |
| **纳管** | 复制到 `~/.skills-manager/`，原位置变 symlink | - | `adopt_ide_skill` |
| **导入** | 复制到 `~/.skills-manager/skills/` | - | `import_local_skill` |

---

## 10. 推荐阅读顺序

如果你第一次接手这个项目，建议按下面顺序读代码：

1. `package.json`
2. `README.md`
3. `src/main.ts`
4. `src/App.vue`
5. `src/composables/useSkillsManager.ts`
6. `src/composables/useProjectConfig.ts`
7. `src/composables/useIdeConfig.ts`
8. `src-tauri/src/lib.rs`
9. `src-tauri/src/commands/market.rs`
10. `src-tauri/src/commands/skills.rs`

这样你会先拿到：

- 启动链路
- UI 组织方式
- 核心业务流
- 原生命令边界

---

## 11. 当前代码状态下值得注意的事实

这是一些对开发很重要、但容易只看 README 时忽略的点：

### 11.1 当前代码实现偏 OpenCode-first

`src/composables/constants.ts` 中当前默认 IDE 配置只有：

- `OpenCode -> .config/opencode/skills`

这说明当前实现虽然保留了"多 IDE 管理器"的产品方向，但代码默认行为仍然是 **OpenCode MVP 优先**。

### 11.2 `App.vue` 目前比较"胖"

它既负责：

- 顶部导航
- 初始化
- 多个 composable 装配
- 多个 modal 状态
- 多条业务流程入口

所以后续如果页面复杂度继续增加，`App.vue` 会是一个自然的拆分候选。

### 11.3 `useSkillsManager.ts` 是主要复杂度集中点

这个文件承担了大量横切职责，是目前最核心、也是最容易继续膨胀的模块。

如果后续要做较大演进，最值得优先拆的通常是它。

### 11.4 IDE 浏览页面默认只显示 OpenCode

虽然 README 列出了很多 IDE（Claude、Cursor、VSCode 等），但：
- `defaultIdeOptions` 中只有 OpenCode
- `scan_overview` 命令中硬编码了多个 IDE 路径用于扫描
- 但 UI 上默认只显示 OpenCode 按钮

这意味着：
- 扫描时会检测多个 IDE 的 skills
- 但用户只能在 UI 上看到 OpenCode 的筛选按钮
- 要看其他 IDE 的 skills，需要手动添加自定义 IDE

### 11.5 Skill 安装的核心是符号链接

很多用户可能误解"安装"是复制文件，实际上：
- **安装** = 创建符号链接（symlink/junction）
- **卸载** = 删除符号链接（不删除实际文件）
- **纳管** = 复制到 manager + 创建符号链接替换原文件
- **删除** = 只删除 manager 目录中的实际文件

理解这一点对排查问题很重要。

---

## 12. 一句话总结

如果要用一句话概括这个项目：

> `skills-manager` 是一个以 `App.vue + useSkillsManager + Tauri commands` 为主轴的桌面技能管理器；前端负责流程与状态组织，Rust 负责本地系统能力，项目当前实现重心偏向 OpenCode 场景。

如果你要开始改功能，默认优先检查这三处：

1. `src/App.vue`
2. `src/composables/useSkillsManager.ts`
3. `src-tauri/src/commands/*.rs`
