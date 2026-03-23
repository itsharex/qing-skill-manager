# Skill版本管理系统设计方案

## 1. 当前系统分析

### 1.1 现状问题

**冲突检测过于简单**
- 仅对比名称、描述、路径三个字段
- 没有内容级别的diff比较
- 无法识别实质性的代码差异

```rust
// 当前冲突检测逻辑 (skills.rs:762-771)
let status = if let Some(existing) = existing_names.get(&name) {
    if existing.description == description && existing.path == path.display().to_string() {
        ProjectSkillImportStatus::Duplicate
    } else {
        ProjectSkillImportStatus::Conflict
    }
} else {
    ProjectSkillImportStatus::New
};
```

**无版本管理概念**
- 同名skill只能存在一个
- 强制重命名破坏原有语义
- 无法追踪skill演变历史

**冲突解决选项有限**
- Keep: 完全忽略新内容
- Overwrite: 丢失旧版本信息  
- Coexist: 简单重命名，缺乏语义

### 1.2 存储结构现状
```
~/.skills-manager/skills/
├── skill-a/
│   └── SKILL.md
├── skill-b/
│   └── SKILL.md
└── ...
```

## 2. 基于Git理念的版本管理设计

### 2.1 核心概念映射

| Git概念 | Skill版本管理 | 实现方式 |
|---------|--------------|----------|
| Repository | Skill Package | 同名skill的集合 |
| Commit | Version | 带version tag的skill内容 |
| Branch | Variant | 同名skill的不同变体 |
| Hash | Content Hash | 文件内容的SHA256 |
| Tag | Version Label | 语义化版本号或自定义名称 |
| Diff | Content Diff | 两个版本的差异对比 |

### 2.2 新的数据结构

```typescript
// Skill版本标识
interface SkillVersion {
  id: string;                    // 版本唯一ID (content_hash)
  skillId: string;               // 技能包ID (name_namespace)
  version: string;               // 语义化版本号 x.y.z
  displayName: string;           // 自定义显示名称
  contentHash: string;           // 内容哈希 (SHA256 of SKILL.md)
  createdAt: number;             // 创建时间
  source: 'market' | 'project' | 'import' | 'clone';
  sourceUrl?: string;            // 来源URL
  parentVersion?: string;        // 父版本ID (支持版本追溯)
  isActive: boolean;             // 是否激活状态
  metadata: {
    name: string;
    description: string;
    author?: string;
    namespace?: string;
  };
}

// 技能包 (同名skill的集合)
interface SkillPackage {
  id: string;                    // name_namespace
  name: string;
  namespace: string;
  defaultVersion: string;        // 默认版本ID
  versions: SkillVersion[];      // 所有版本
  variants: SkillVariant[];      // 分支/变体
}

// 变体 (类似git branch)
interface SkillVariant {
  id: string;
  name: string;                  // 变体名称 (e.g., "stable", "dev", "custom")
  currentVersion: string;        // 当前指向的版本
  createdAt: number;
  description?: string;
}

// 冲突检测 (Git-style)
interface SkillConflict {
  type: 'content' | 'structure' | 'metadata' | 'dependency';
  severity: 'minor' | 'major' | 'breaking';
  baseVersion: SkillVersion;
  incomingVersion: SkillVersion;
  diff: SkillDiff;
  autoResolvable: boolean;
}

// 差异对比
interface SkillDiff {
  filesChanged: string[];
  additions: number;
  deletions: number;
  contentDiff: {
    before: string;
    after: string;
    unified: string;             // unified diff格式
  };
  metadataChanges: {
    field: string;
    oldValue: unknown;
    newValue: unknown;
  }[];
}
```

### 2.3 新的存储结构

```
~/.skills-manager/
├── skills/
│   └── packages/
│       ├── skill-a_namespace-a/
│       │   ├── metadata.json          # SkillPackage元数据
│       │   ├── versions/
│       │   │   ├── v1.0.0_a1b2c3d/    # 版本内容 (hash命名)
│       │   │   │   └── SKILL.md
│       │   │   ├── v1.1.0_e4f5g6h/
│       │   │   │   └── SKILL.md
│       │   │   └── v2.0.0_i7j8k9l/
│       │   │       └── SKILL.md
│       │   └── variants/
│       │       ├── stable -> ../versions/v1.0.0_a1b2c3d   # symlink
│       │       ├── latest -> ../versions/v2.0.0_i7j8k9l
│       │       └── custom-variant -> ../versions/v1.1.0_e4f5g6h
│       └── skill-b_namespace-b/
│           └── ...
├── index.json                         # 全局索引
└── versions/                           # 版本历史
    └── skill-a_namespace-a/
        └── history.json
```

## 3. 新的展示和模块功能

### 3.1 Skill Package 展示

**列表视图增强**
- 显示版本数量徽章
- 显示最新版本号
- 显示变体数量
- 显示最后更新时间

**详情面板 (新增)**
```
┌─────────────────────────────────────────────────────┐
│  📦 skill-name                              [v]更多 │
│  namespace: skill-author                            │
├─────────────────────────────────────────────────────┤
│  版本历史                                            │
│  ├── v2.0.0  (latest)     2024-03-20  feature        │
│  ├── v1.1.0               2024-02-15  bugfix         │
│  └── v1.0.0  (stable)     2024-01-01  initial        │
├─────────────────────────────────────────────────────┤
│  变体 (Variants)                                     │
│  ├── stable  → v1.0.0                               │
│  ├── latest  → v2.0.0                               │
│  └── dev     → v2.1.0-beta                          │
├─────────────────────────────────────────────────────┤
│  安装目标                                            │
│  [ ] VSCode   [ ] Cursor   [ ] Claude               │
├─────────────────────────────────────────────────────┤
│  [安装此版本]  [创建变体]  [对比版本]  [删除版本]      │
└─────────────────────────────────────────────────────┘
```

### 3.2 版本管理功能

**版本创建**
- 自动版本号生成 (基于语义化版本)
- 自定义版本名称
- 从现有skill创建新版本
- 克隆现有版本作为新变体

**版本操作**
- 设置默认版本
- 版本间切换
- 版本对比 (diff视图)
- 版本回滚
- 版本标签管理

**变体管理**
- 创建变体
- 重命名变体
- 切换变体指向的版本
- 删除变体

**版本删除**
```typescript
interface DeleteVersionOptions {
  versionId: string;
  strategy: 'soft' | 'hard' | 'archive';
  // soft: 标记删除，保留文件
  // hard: 完全删除
  // archive: 移动到归档目录
  force?: boolean;  // 强制删除，忽略依赖检查
}
```

### 3.3 冲突解决升级 (Git-style)

**智能冲突检测**
```typescript
enum ConflictType {
  IDENTICAL = 'identical',           // 完全相同的版本
  PATCH = 'patch',                   // 小修补 (metadata变更)
  MINOR = 'minor',                   // 次要变更 (新增功能)
  MAJOR = 'major',                   // 主要变更 (破坏性变更)
  FORK = 'fork',                     // 实质性分歧 (类似git diverged)
}

interface ConflictAnalysis {
  type: ConflictType;
  similarity: number;                // 0-1 相似度评分
  differences: Difference[];
  suggestions: ResolutionSuggestion[];
}
```

**可视化对比界面**
```
┌───────────────────────────────────────────────────────┐
│ 版本对比: skill-name                                  │
│ v1.0.0 (stable)  ←→  v1.1.0 (latest)                 │
├───────────────────────────────────────────────────────┤
│                                                       │
│  相似度: 87%  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░                 │
│                                                       │
│  变更概览:                                            │
│  • 文件变更: 1                                        │
│  • 新增行: 45                                         │
│  • 删除行: 12                                         │
│  • 元数据变更: description, author                   │
│                                                       │
├───────────────────────────────────────────────────────┤
│ [并排对比] [统一差异] [仅变更]                        │
├───────────────────────────────────────────────────────┤
│ diff --skill a/SKILL.md b/SKILL.md                   │
│ @@ -1,5 +1,8 @@                                      │
│  ---                                                 │
│  name: skill-name                                    │
│ -version: 1.0.0                                      │
│ +version: 1.1.0                                      │
│  description: Original description                   │
│ +description: Enhanced description with more...     │
│  ---                                                 │
│  + New feature added here                            │
│  + More documentation                                │
├───────────────────────────────────────────────────────┤
│ [作为新版本添加] [覆盖现有] [创建变体] [放弃]        │
└───────────────────────────────────────────────────────┘
```

**解决策略选项**
- **Fast-forward**: 简单前进 (无冲突时)
- **Merge**: 智能合并 (尝试自动合并)
- **Create Variant**: 创建新变体保留两者
- **Interactive Merge**: 交互式解决冲突

## 4. 技术可行性评估

### 4.1 实现复杂度

| 模块 | 复杂度 | 工作量 | 依赖 |
|------|--------|--------|------|
| 版本存储结构重构 | 高 | 3-4天 | 需要迁移脚本 |
| 内容哈希计算 | 低 | 0.5天 | 使用Rust crypto库 |
| Diff算法 | 中 | 1-2天 | diff crate |
| 版本管理UI | 高 | 4-5天 | 需要新组件 |
| 冲突检测升级 | 中 | 2-3天 | 依赖diff算法 |
| 变体系统 | 中 | 2天 | symlink管理 |
| 迁移工具 | 中 | 1-2天 | 数据迁移 |

**总工作量**: 约 14-18 天

### 4.2 技术方案

**Rust后端 (Tauri Commands)**
```rust
// 版本管理命令
#[tauri::command]
pub fn create_skill_version(
    request: CreateVersionRequest
) -> Result<SkillVersion, String> { }

#[tauri::command]
pub fn compare_skill_versions(
    request: CompareVersionsRequest
) -> Result<SkillDiff, String> { }

#[tauri::command]
pub fn delete_skill_version(
    request: DeleteVersionRequest
) -> Result<(), String> { }

#[tauri::command]
pub fn set_default_version(
    request: SetDefaultVersionRequest
) -> Result<(), String> { }

#[tauri::command]
pub fn create_skill_variant(
    request: CreateVariantRequest
) -> Result<SkillVariant, String> { }

// 增强冲突检测
#[tauri::command]
pub fn analyze_skill_conflict(
    request: AnalyzeConflictRequest
) -> Result<ConflictAnalysis, String> { }
```

**Vue前端组件**
```typescript
// 新增 composables
useSkillVersionManager()      // 版本管理
useSkillVariantManager()      // 变体管理  
useSkillDiffViewer()          // 对比查看
useSkillConflictResolver()    // 冲突解决

// 新增组件
SkillPackageCard.vue          // 技能包卡片
SkillVersionList.vue          // 版本列表
SkillDiffViewer.vue           // 差异对比
SkillVariantManager.vue       // 变体管理
VersionSelector.vue           // 版本选择器
```

### 4.3 数据迁移策略

**迁移脚本** (migration-v1-to-v2)
1. 扫描现有skills目录
2. 为每个skill创建Package结构
3. 计算内容哈希生成版本ID
4. 创建默认变体 (stable)
5. 生成metadata.json
6. 保留原有数据作为备份

```rust
pub fn migrate_v1_to_v2() -> Result<MigrationReport, MigrationError> {
    // 1. 备份现有数据
    // 2. 读取所有现有skills
    // 3. 按名称分组创建packages
    // 4. 生成版本信息
    // 5. 创建新的存储结构
    // 6. 验证迁移结果
    // 7. 生成迁移报告
}
```

## 5. 实施计划

### Phase 1: 基础架构 (Week 1)
- [ ] 设计新的数据结构和存储格式
- [ ] 实现内容哈希计算
- [ ] 重构存储目录结构
- [ ] 实现版本元数据管理
- [ ] 编写数据迁移脚本

### Phase 2: 版本管理核心 (Week 2)
- [ ] 版本CRUD操作
- [ ] 变体系统实现
- [ ] 默认版本管理
- [ ] 版本选择逻辑
- [ ] Tauri commands实现

### Phase 3: 对比与冲突 (Week 3)
- [ ] Diff算法实现
- [ ] 冲突分析引擎
- [ ] 增强的冲突检测
- [ ] 智能合并建议
- [ ] 对比API

### Phase 4: UI/UX (Week 4)
- [ ] Skill Package卡片组件
- [ ] 版本列表和详情
- [ ] 差异对比视图
- [ ] 变体管理界面
- [ ] 冲突解决流程
- [ ] 版本删除确认

### Phase 5: 集成与优化 (Week 5)
- [ ] 与现有功能集成
- [ ] 向后兼容处理
- [ ] 性能优化
- [ ] 错误处理完善
- [ ] 测试覆盖

### Phase 6: 发布准备
- [ ] 完整测试
- [ ] 文档更新
- [ ] 迁移指南
- [ ] 版本发布

## 6. 关键设计决策

### 6.1 版本号策略
- 采用语义化版本 (SemVer): MAJOR.MINOR.PATCH
- 自动检测版本增量类型 (基于diff分析)
- 允许自定义版本名称作为别名

### 6.2 存储优化
- 版本内容使用hash命名去重
- 相同内容只存储一次
- 变体使用symlink指向版本
- 支持垃圾回收删除未引用版本

### 6.3 向后兼容
- 保留原有API作为兼容层
- 现有skills自动迁移到新结构
- 提供降级方案

### 6.4 性能考虑
- 延迟加载版本列表
- 缓存diff结果
- 异步计算内容哈希
- 索引加速查找

## 7. 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 数据迁移失败 | 中 | 高 | 完整备份，可回滚 |
| 存储空间增加 | 低 | 中 | 内容去重，垃圾回收 |
| 性能下降 | 低 | 中 | 缓存，异步，索引 |
| 复杂度增加 | 高 | 中 | 清晰的架构，文档 |
| 用户学习成本 | 中 | 中 | 直观的UI，引导 |

## 8. 结论

**✅ 实现可行**

这个设计在技术上完全可以实现，基于现有的Tauri+Vue架构，通过合理的数据结构和增量开发，可以实现一个强大的Git-style skill版本管理系统。

**关键优势:**
1. 真正的版本管理，不再只是重命名
2. 智能冲突检测，基于内容而非仅元数据
3. 变体系统支持多场景需求
4. 可视化对比工具
5. 灵活的版本删除策略

**建议:**
- 采用增量开发，先实现核心版本管理
- 保持向后兼容，平滑迁移
- 优先考虑用户体验，避免过度复杂

你希望我什么时候开始实现？我可以从Phase 1开始，或者你有其他优先级要求？
