# Qing Skill Manager — 分模块代码审查提示词

> 使用方式：将以下提示词分别发送给 Claude Code / AI，每次聚焦一个模块进行审查。
> 建议按顺序执行：核心类型 → 后端 → Composables → 组件 → 国际化 → 测试。

---

## 1. 核心类型与数据模型

```
请审查 skills-manager 项目的类型系统，涉及以下文件：

- src/composables/types.ts（前端类型定义）
- src-tauri/src/types.rs（后端类型定义）

审查重点：
1. 前后端类型是否一一对应、字段命名是否一致（Rust 用 camelCase serde）
2. 是否存在语义重叠的类型（如 LocalSkill / SkillVersion / ProjectSkill），能否合并或用判别联合（discriminated union）统一
3. 可选字段是否合理，是否有应该必填却标为可选的字段
4. 类型命名是否清晰表达其用途，是否有歧义
5. 是否缺少关键的类型定义（如错误类型、操作状态枚举）

请给出具体的重构建议和示例代码。
```

---

## 2. Rust 后端 — 技能扫描与安装

```
请审查 skills-manager 的 Rust 后端技能管理模块：

- src-tauri/src/commands/skills/mod.rs（共享工具函数，约 1900 行）
- src-tauri/src/commands/skills/scan.rs（扫描、安装、卸载、收录）

审查重点：
1. mod.rs 近 2000 行，是否应该拆分？哪些函数可以提取到独立模块？
2. SKILL.md 元数据解析逻辑是否健壮（按行分割、格式假设），有哪些边界情况会失败？
3. clone_local_skill 安装流程：是否在复制前验证了源文件完整性？sidecar 文件写入失败时如何处理？
4. scan_overview 每次全量扫描的性能影响，是否可以引入增量扫描或缓存机制？
5. 路径处理（canonicalize、安全校验）是否覆盖了所有边界情况（符号链接、空格、Unicode 路径、Windows 保留名）
6. 错误处理：是否所有 unwrap/expect 都是安全的？是否应该用自定义错误类型替代字符串错误？

请给出重构方案，优先级从高到低排列。
```

---

## 3. Rust 后端 — 版本与冲突管理

```
请审查 skills-manager 的版本与冲突管理模块：

- src-tauri/src/commands/skills/version.rs（版本 CRUD、比较、包管理）
- src-tauri/src/commands/skills/conflict.rs（冲突分析与解决）
- src-tauri/src/commands/skills/variant.rs（变体管理）

审查重点：
1. 版本状态持久化（JSON 文件）的读写是否有并发安全问题？多次快速操作是否会导致数据丢失？
2. compare_skill_versions 的 diff 算法实现质量，对大文件的表现如何？
3. 冲突解决的三种策略（keep/overwrite/coexist）是否覆盖了所有实际场景？
4. 软删除（deleted=true）在版本列表展示和重用时的逻辑是否一致？
5. 变体（variant）与版本（version）的关系模型是否清晰，用户能否直观理解？
6. sidecar 文件（.qing-skill-version.json / .qing-skill-manager-version.json）格式是否有版本兼容性设计？

请给出当前最需要修复的 3 个问题及其解决方案。
```

---

## 4. Rust 后端 — 市场与网络

```
请审查 skills-manager 的市场模块：

- src-tauri/src/commands/market.rs（市场搜索、下载、更新）
- src-tauri/src/utils/download.rs（下载工具）
- src-tauri/src/utils/security.rs（安全校验）

审查重点：
1. search_marketplaces 对多种 API 格式的解析（Claude Marketplace / SkillsLLM）是否健壮？JSON 结构不符预期时的容错处理？
2. 下载流程是否有完整性校验（hash 验证）？下载中断后能否恢复？
3. 网络请求是否有合理的超时、重试策略？
4. API Key 的存储和传输是否安全？是否有泄露风险？
5. security.rs 的路径校验是否能防御路径穿越攻击（path traversal）？
6. 是否存在 SSRF 风险（用户可控的 URL 输入）？

请按安全风险等级从高到低列出发现的问题。
```

---

## 5. Composable — useSkillsManager（中央编排器）

```
请审查 skills-manager 的核心 composable：

- src/composables/useSkillsManager.ts（中央编排器，组合 10+ 子 composable）

审查重点：
1. 作为 facade 模式的实现，它的职责边界是否清晰？是否存在"上帝对象"问题？
2. 返回 70+ 属性/方法是否过多？哪些可以按需暴露或拆分到子 composable？
3. 各子 composable 之间的依赖关系是否清晰？是否有循环依赖或隐式依赖？
4. 回调注入模式（notification callback、translate function t）是否是最佳实践？是否有更好的替代方案？
5. busy state 的管理是否可能出现竞态条件（多个异步操作同时进行）？
6. scanLocalSkills() 在几乎每个操作后都被调用，这是否合理？

请提出具体的拆分方案，给出重构前后的对比。
```

---

## 6. Composable — 安装与卸载流程

```
请审查 skills-manager 的安装/卸载相关 composable：

- src/composables/useInstallActions.ts（安装流程）
- src/composables/useUninstallActions.ts（卸载流程）
- src/composables/useIdeAdoption.ts（IDE 技能收录）
- src/composables/useIdeConfig.ts（IDE 配置管理）
- src/composables/constants.ts（IDE 路径映射、存储键）

审查重点：
1. 安装流程的目标路径构建是否正确处理了所有 IDE 的路径格式差异？
2. 安装失败后的回滚机制是否存在？部分复制的文件如何清理？
3. 卸载后是否正确清理了所有关联数据（sidecar 文件、版本追踪记录）？
4. IDE 路径映射表（constants.ts）的扩展性如何？添加新 IDE 是否方便？
5. 自定义 IDE 路径的校验是否充分？
6. adopt 和 import 的语义区分是否在代码中足够清晰？

请重点关注数据一致性问题，给出需要添加的防御性检查。
```

---

## 7. Composable — 项目管理

```
请审查 skills-manager 的项目管理相关 composable：

- src/composables/useProjectConfig.ts（项目 CRUD、持久化）
- src/composables/useProjectHandlers.ts（项目操作处理器）
- src/composables/useProjectScan.ts（项目技能扫描）
- src/composables/useProjectSnapshots.ts（快照刷新）

审查重点：
1. 项目数据仅存储在 localStorage，是否有容量限制和数据丢失风险？
2. useProjectHandlers（385 行）是否混合了 UI 状态管理和业务逻辑？如何分离？
3. 项目扫描对大型项目目录的性能表现如何？是否有防抖或限流？
4. 快照刷新策略是否合理（tab 切换触发）？是否应该改为变更驱动？
5. 项目路径变更（移动、删除）后的状态同步如何处理？
6. 多项目共享同一技能时的一致性如何保证？

请给出最影响用户体验的 3 个问题及其改进方案。
```

---

## 8. Composable — Library Workspace

```
请审查 skills-manager 的 Library 工作区逻辑：

- src/composables/useLibraryWorkspace.ts（库工作区状态派生，445 行）
- src/composables/useVersionManagement.ts（版本管理操作，308 行）

审查重点：
1. useLibraryWorkspace 的计算属性链是否有不必要的重计算？能否用 shallowRef 或 computed 缓存优化？
2. 过滤和搜索逻辑（平台过滤、关键词搜索）的性能，技能数量增长后是否会卡顿？
3. useVersionManagement 和 useLibraryWorkspace 之间是否有职责重叠？
4. 版本比较状态在 App.vue 和 composable 中是否有重复管理？
5. 三栏布局（sidebar/detail/version rail）的数据流是否单向清晰？
6. 选中状态（selectedSkill、selectedVersion）的管理是否可能出现不一致？

请重点分析计算属性的依赖图，找出可优化的性能热点。
```

---

## 9. Vue 组件 — App.vue 与面板组件

```
请审查 skills-manager 的主要 Vue 组件：

- src/App.vue（根组件，345 行 script setup）
- src/components/MarketPanel.vue（339 行）
- src/components/IdePanel.vue（384 行）
- src/components/SettingsPanel.vue（473 行）
- src/components/ProjectsPanel.vue

审查重点：
1. App.vue 的 script setup 过长（345 行），哪些逻辑应该提取到 composable？
2. 各面板组件的 props 传递深度是否过深？是否应该引入 provide/inject 或状态管理？
3. 事件冒泡链是否过长（子组件 → 面板 → App.vue → composable）？
4. 各面板是否有共同的模式可以抽象（如 loading/error/empty 状态处理）？
5. SettingsPanel 的表单状态管理是否合理？
6. 组件是否正确使用了 v-if/v-show 进行条件渲染以避免不必要的挂载？

请给出 App.vue 瘦身方案，说明提取前后的代码对比。
```

---

## 10. Vue 组件 — Modal 组件群

```
请审查 skills-manager 的 Modal 组件群：

- src/components/VersionManagerModal.vue（1439 行，最大组件）
- src/components/ConflictResolutionModal.vue（597 行）
- src/components/ProjectSkillImportModal.vue（566 行）
- src/components/ImportToProjectModal.vue（463 行）
- src/components/InstallModal.vue（353 行）
- src/components/VersionDiffModal.vue（389 行）

审查重点：
1. VersionManagerModal 1439 行严重过长，如何拆分为多个子组件？
2. 各 Modal 的打开/关闭状态管理是否统一？是否应该抽取 useModalState composable？
3. Modal 之间的数据传递（如从 VersionManager 打开 VersionDiff）是否清晰？
4. 表单验证是否完整？用户输入是否有即时反馈？
5. Modal 的无障碍访问（焦点管理、键盘导航、ESC 关闭）是否实现？
6. 各 Modal 是否有共同的 UI 模式（标题栏、操作按钮、loading 状态）可以抽象为 BaseModal？

请给出 VersionManagerModal 的拆分方案，包括子组件划分和数据流设计。
```

---

## 11. Library 工作区组件

```
请审查 skills-manager 的 Library 工作区组件：

- src/components/library/LibraryWorkspace.vue
- src/components/library/LibrarySidebar.vue
- src/components/library/LibraryDetailPanel.vue
- src/components/library/LibraryVersionRail.vue

审查重点：
1. 三栏布局的响应式设计是否完善？窗口尺寸变化时的行为是否正确？
2. 组件间的 props/events 通信是否最优？是否有适合 provide/inject 的场景？
3. 列表渲染（sidebar 技能列表、version rail）是否使用了 key 属性和虚拟滚动？
4. 选中状态的同步（sidebar 选择 → detail 更新 → version rail 更新）是否流畅无闪烁？
5. 空状态和加载状态的 UI 处理是否一致？
6. CSS 变量的命名和使用是否遵循了项目的主题系统规范？

请重点关注用户交互流畅度相关的问题。
```

---

## 12. 国际化（i18n）

```
请审查 skills-manager 的国际化实现：

- src/locales/zh-CN.ts（21KB）
- src/locales/en-US.ts（21KB）
- src/composables/usePreferences.ts（语言切换）

审查重点：
1. 两份语言文件是否完全同步？是否有一方遗漏的 key？
2. 翻译文本质量：英文翻译是否自然？是否有直译问题？
3. 是否有硬编码在组件模板或 JS 中的文本未走 i18n？
4. 是否有未使用的翻译 key（死代码）？
5. 文件结构是否可以按功能模块拆分以降低维护成本？
6. 是否处理了日期、数字、文件路径的本地化格式？
7. t() 调用是否都传递了必要的插值参数？

请给出一个脚本思路，用于自动检测两份 locale 文件的 key 差异。
```

---

## 13. 测试覆盖率

```
请审查 skills-manager 的测试现状并给出改进建议：

现有测试文件：
- src/composables/constants.test.mjs
- src/composables/utils.test.mjs
- src/composables/useLibraryWorkspace.test.mjs
- src/composables/useProjectConfig.test.mjs
- src-tauri/src/ 下的 #[cfg(test)] 模块

审查重点：
1. 当前测试覆盖了哪些关键路径？哪些高风险逻辑完全没有测试？
2. 前端测试使用手动断言（无框架），是否应该迁移到 Vitest？利弊分析？
3. 以下场景是否需要优先补充测试？按优先级排列：
   - 安装/卸载流程的路径构建
   - 版本比较逻辑
   - 冲突解决策略
   - 市场搜索结果解析
   - localStorage 序列化/反序列化
4. Rust 后端哪些函数最需要单元测试？
5. 是否需要前后端集成测试？如何实现？
6. 是否需要 E2E 测试？推荐什么工具？

请给出一个分阶段的测试补全计划，包括每阶段的目标覆盖率。
```

---

## 14. 整体架构与性能

```
请对 skills-manager 进行整体架构审查：

项目概述：Tauri 2 + Vue 3 + TypeScript 桌面应用，管理 AI 编程助手的技能文件。
当前版本：v0.3.25

审查重点：

架构问题：
1. 无集中状态管理（纯 composable + refs），在当前规模下是否已到达需要引入 Pinia 的临界点？
2. App.vue 作为唯一的编排层，职责是否过重？是否需要路由或子页面拆分？
3. 前后端通信全部通过 invoke()，是否有需要改用 Tauri event 系统（推送模式）的场景？

性能问题：
4. scanLocalSkills() 在几乎每个操作后全量执行，随技能数量增长的性能预期？
5. 市场搜索无分页时大量结果的渲染性能？
6. 版本 diff 对大文件的处理能力？
7. 多个 watch/computed 级联触发是否有不必要的重计算？

可扩展性：
8. 当前架构能否支撑以下未来需求？需要哪些改造？
   - 技能依赖关系管理
   - 技能模板/生成器
   - 团队协作与技能共享
   - 技能使用统计与推荐

请给出一份优先级排序的技术债务清单，标注每项的影响范围和建议修复时机。
```

---

## 使用建议

| 审查顺序 | 模块 | 预估耗时 | 优先级 |
|---------|------|---------|-------|
| 1 | 核心类型（#1） | 15 min | 高 — 类型是一切的基础 |
| 2 | Rust 技能扫描（#2） | 30 min | 高 — 最大模块，风险最高 |
| 3 | useSkillsManager（#5） | 20 min | 高 — 前端核心 |
| 4 | Modal 组件群（#10） | 25 min | 高 — 用户交互核心 |
| 5 | 版本与冲突（#3） | 20 min | 中 — 复杂业务逻辑 |
| 6 | 市场与网络（#4） | 15 min | 中 — 安全相关 |
| 7 | 安装卸载（#6） | 15 min | 中 — 数据一致性 |
| 8 | 项目管理（#7） | 15 min | 中 |
| 9 | Library Workspace（#8, #11） | 20 min | 中 |
| 10 | App.vue 与面板（#9） | 20 min | 低 — 依赖前面的重构 |
| 11 | 国际化（#12） | 10 min | 低 |
| 12 | 测试覆盖（#13） | 15 min | 中 — 贯穿所有模块 |
| 13 | 整体架构（#14） | 30 min | 高 — 战略方向 |
