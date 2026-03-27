# Qing Skill Manager v0.3.25 — 代码审查与优化建议总报告

> 审查时间：2026-03-27 | 审查范围：全项目（Rust 后端 + Vue 前端 + 类型系统 + 测试）

---

## 一、Bug 审查（共 23 项）

### Critical（3 项）

| ID | 问题 | 文件 | 影响 |
|----|------|------|------|
| BUG-01 | `search_marketplaces` 中 SkillsLLM 对查询参数重复编码 | `market.rs:266` | 特殊字符搜索结果不一致 |
| BUG-02 | `download_marketplace_skill` 缺少 SSRF 防护，未校验 URL schema 和目标主机 | `market.rs:415-436`, `download.rs:33-99` | 恶意市场可触发内网请求 |
| BUG-03 | `uninstall_skill` 路径校验可被符号链接绕过 | `scan.rs:235-293` | 可能删除 allowed roots 之外的目录 |

### High（9 项）

| ID | 问题 | 文件 |
|----|------|------|
| BUG-04 | `version_summary_for_skill` 用 `let _ =` 吞掉写入错误 | `mod.rs:366-370` |
| BUG-05 | `collect_skills_from_dir` 每次调用都写入文件系统，scan 路径有副作用 | `mod.rs:475-511` |
| BUG-06 | `delete_skill_version` Archive 策略不检查目标目录是否已存在 | `version.rs:239-254` |
| BUG-07 | `adopt_ide_skill` 三步非原子操作，remove 后 copy 失败会丢数据 | `scan.rs:325-382` |
| BUG-08 | `scan_overview` 中 N+1 查询问题（O(N^2) 文件操作） | `scan.rs:78-232` |
| BUG-09 | sidecar 不记录安装来源版本，版本切换后同步状态误判 | `scan.rs:59-65` |
| FLOW-01 | **`resolve_skill_conflict` 的 Overwrite/Coexist 策略不执行任何文件操作** | `conflict.rs:47-85` |
| FLOW-02 | 项目级技能卸载因 `projectDir: null` 硬编码可能失败 | `useUninstallActions.ts:73` |
| FLOW-03 | Adopt 后不写入 install sidecar，scan 时显示 untracked | `scan.rs:375-376` |

### Medium（8 项）

| ID | 问题 | 文件 |
|----|------|------|
| BUG-10 | `sanitize_dir_name` 对纯中文名称全部生成 "skill"，导致冲突 | `path.rs:61-95` |
| BUG-11 | `import_local_skill` 已存在时返回 Ok 但不更新内容 | `scan.rs:296-322` |
| BUG-12 | 前后端安全校验不一致（前端检查更严格但可绕过） | `utils.ts:34-53`, `security.rs:3-19` |
| BUG-13 | SkillsMP 未启用时状态显示为 NeedsKey 而非 Disabled | `market.rs:311` |
| BUG-14 | `delete_local_skills` 批量删除部分失败时无法回滚或报告 | `scan.rs:385-415` |
| BUG-15 | `build_skill_version` 每次扫描都更新 `created_at`，时间无意义 | `mod.rs:277-307` |
| BUG-16 | `content_hash` 仅基于 SKILL.md，不含 skill 目录其他文件 | `mod.rs:245-252` |
| BUG-17 | 批量安装部分失败时错误信息不含部分成功详情 | `useInstallActions.ts:191-212` |

### Low（3 项）

| ID | 问题 | 文件 |
|----|------|------|
| BUG-18 | SKILL.md frontmatter 不处理 YAML 引号 | `mod.rs:160-208` |
| BUG-19 | ZIP 解压无文件数量限制 | `download.rs:126-166` |
| BUG-20 | `resolve_canonical` 行为不一致导致路径比较非确定性 | `path.rs:115-119` |

---

## 二、代码架构问题（9 项）

### P0 — 立即修复（低成本高收益）

**1. 回调参数重复注入**
- `ToastFunction` / `TranslateFunction` / `ScanLocalSkillsFunction` 在 **7 个文件**中重复声明
- `useSkillsManager` 中重复传入 `(msg) => toast.success(msg)` 至少 6 次
- **方案**：提取 `AppContext` 使用 `provide/inject`，消除所有重复

**2. App.vue 直接调用 invoke() 绕过 composable 层**
- `App.vue:220,248` 直接调用 `invoke("import_local_skill")` 和 `invoke("adopt_ide_skill")`
- 9 个 async handler 应移入对应 composable
- **方案**：App.vue 只做组合、绑定和生命周期，不含业务逻辑

### P1 — 短期修复

**3. BaseModal 抽象缺失**
- 所有 Modal 独立实现 Teleport + overlay + header + close，约 200 行重复 CSS/模板
- **方案**：抽取 `BaseModal.vue`，统一交互行为和样式

**4. useSkillsManager 是"上帝对象"**
- 返回 **120+ 属性**，App.vue 有 97 行解构
- **方案**：按领域分组导出（`market`、`library`、`install`、`version`、`project` 等命名空间）

### P2 — 中期重构

**5. VersionManagerModal 过大（1439 行）**
- 内聚 7 个子功能：版本列表、重命名、删除、对比、创建、项目选择、Variant
- **方案**：拆为容器组件（~200 行）+ 6 个子组件

**6. LibraryWorkspace props drilling + 重复计算**
- 10 个 props，18 个 emits，逐层透传
- `useLibraryWorkspace.ts` 和 `LibraryWorkspace.vue` 独立计算相同数据
- **方案**：使用 `provide/inject` 替代 props 链

### P3 — 低优先级

**7. 事件冒泡链过长**（4 层冒泡：VersionRail → Workspace → App → composable）
**8. 前后端类型不一致**（Rust 用 String，TS 用 union type）
**9. 语义重叠类型**（LibrarySkill vs LocalSkill 同源不同视图）

### 关于 Pinia

**结论：当前不需要。** 问题本质是组合层设计（扁平化导出 + 缺少 provide/inject），不是状态管理方案选择。

---

## 三、核心功能检查（6 个流程）

### 流程健康度总览

| 流程 | 健康度 | 关键风险 |
|------|--------|---------|
| 技能安装/卸载 | 中 | 无回滚、项目级卸载失败、安装已存在直接跳过 |
| 版本管理 | 低 | hash 只覆盖 SKILL.md、删除 default 不自动切换、scan 有写副作用 |
| 市场搜索下载 | 中 | SSRF 风险、分页合并不精确、缓存 key 不含市场状态 |
| **冲突解决** | **极低** | **Overwrite/Coexist 策略是空操作，不执行文件操作** |
| 项目管理 | 中 | 路径变更无处理、按名称去重可能丢数据、30 秒轮询 I/O |
| IDE 收录 (Adopt) | 低 | 非原子操作可能丢数据、不写 sidecar、名称冲突静默覆盖 |

### 最需修复的 3 个功能缺陷

1. **冲突解决是空操作** — `resolve_skill_conflict` 返回成功但不执行任何文件复制/覆盖，用户以为冲突已解决但实际上没有
2. **Adopt 数据丢失风险** — 先删后复制的非原子操作，copy 失败时原始文件已删
3. **版本 hash 不可靠** — 只 hash SKILL.md 内容，多文件技能的变更检测无效

---

## 四、下一步优化路线图

### 短期（1-2 周）— 投入小、收益大

| # | 内容 | 耗时 | 优先级 |
|---|------|------|--------|
| S1 | 添加 ESLint + Prettier + pre-commit hook | 2h | 5 |
| S2 | 添加 CI 测试 workflow（PR 时运行 `pnpm verify`） | 1h | 5 |
| S3 | 修复冲突解决空操作 BUG（FLOW-01） | 4h | 5 |
| S4 | 修复 Adopt 非原子操作（BUG-07）— 先 copy 验证再 remove | 2h | 5 |
| S5 | 修复 SSRF 防护（BUG-02）— 校验 URL schema 和主机 | 1h | 5 |
| S6 | 项目快照并行刷新（`Promise.all` 替代串行 for） | 30min | 4 |
| S7 | Error toast 增加手动关闭和复制功能 | 1h | 4 |
| S8 | Modal 无障碍增强（role/aria/ESC/focus trap） | 2h | 4 |
| S9 | 修复 `sanitize_dir_name` 中文名称冲突（BUG-10） | 1h | 4 |
| S10 | i18n 混合语言修复 | 30min | 3 |

### 中期（1-2 月）— 架构改进

| # | 内容 | 耗时 | 优先级 |
|---|------|------|--------|
| M1 | **增量扫描替代全量扫描**（`notify` crate 或 etag 缓存） | 1-2 周 | 5 |
| M2 | **引入 Vitest**，补测核心 composable | 1 周 | 5 |
| M3 | **分离 scan 读写逻辑**（BUG-04/05/15）— scan 只读不写 | 3-5 天 | 5 |
| M4 | **provide/inject 重构**（消除回调重复 + props drilling） | 3-5 天 | 4 |
| M5 | **VersionManagerModal 拆分**（1439 行 → 容器 + 6 子组件） | 3-5 天 | 4 |
| M6 | **content_hash 覆盖全目录**（BUG-16） | 2-3 天 | 4 |
| M7 | **操作撤销/回收站机制**（先移到 trash 再确认删除） | 3-5 天 | 4 |
| M8 | **useSkillsManager 命名空间化导出** | 3-5 天 | 3 |
| M9 | **Library 虚拟滚动**（`vue-virtual-scroller`） | 2-3 天 | 3 |
| M10 | **IDE 配置从硬编码改为配置文件驱动** | 3-5 天 | 3 |

### 长期（3-6 月）— 新功能方向

| # | 内容 | 耗时 | 优先级 |
|---|------|------|--------|
| L1 | 技能创建向导（scaffold 模板） | 1 周 | 4 |
| L2 | 团队技能共享（Git 仓库作为技能源） | 3-4 周 | 4 |
| L3 | 技能依赖关系管理 | 2-3 周 | 3 |
| L4 | 插件化 IDE 适配器接口 | 2-3 周 | 3 |
| L5 | 技能使用统计与推荐 | 2 周 | 2 |
| L6 | E2E 测试覆盖 | 3-4 周 | 2 |

---

## 五、测试现状

| 指标 | 现状 |
|------|------|
| 前端断言数 | ~69（仅辅助函数） |
| 核心业务测试 | 0（下载、安装、版本、冲突均未测试） |
| 测试框架 | 原生 `node:assert`（无 Vitest） |
| Rust 测试 | 有 inline `#[cfg(test)]` |
| CI 集成 | 无（仅本地手动运行） |
| E2E 测试 | 无 |

**优先补测模块**：useDownloadQueue > useInstallActions > useMarketplaceSearch > useVersionManagement

---

## 六、核心结论

1. **最紧急**：冲突解决空操作（FLOW-01）、Adopt 数据丢失风险（BUG-07）、SSRF 漏洞（BUG-02）— 这三个需要立即修复
2. **最大性能瓶颈**：`scan_overview` 全量扫描 + O(N^2) 文件操作 + scan 路径有写副作用
3. **架构核心问题**：不是技术选型问题（composable 模式可行），而是组合层设计——`useSkillsManager` 的 120+ 扁平导出和缺少 `provide/inject`
4. **技术债务优先级**：安全修复 > 数据完整性 > 性能优化 > 架构重构 > 新功能
