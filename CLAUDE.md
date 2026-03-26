# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
pnpm install              # Install dependencies
pnpm tauri dev            # Start full dev environment (Vite + Tauri)
pnpm dev                  # Frontend-only dev server (port 1420)

pnpm build                # Type-check + Vite production build
pnpm tauri build          # Build desktop app bundle

pnpm verify               # Full verification: check + test (strictest)
pnpm check                # Static analysis (frontend typecheck + cargo check/clippy)
pnpm test                 # All tests (frontend logic + typecheck + build verify + cargo test)
pnpm test:frontend:logic  # Run frontend logic tests only (fastest)
pnpm test:rust            # Run Rust tests only (cd src-tauri && cargo test)
pnpm typecheck            # vue-tsc --noEmit
pnpm check:rust:fix       # cargo clippy auto-fix
```

Frontend logic tests are plain Node.js `.mjs` files with manual assertions (no test framework). Run individual tests with `node src/composables/constants.test.mjs`. Rust tests are inline `#[cfg(test)]` modules.

## Architecture

**Tauri 2 desktop app**: Vue 3 + TypeScript frontend, Rust backend. No centralized store — state is managed via Vue 3 composables using the Composition API.

### Data Flow

```
Vue component → composable action → invoke("tauri_command", { request: {...} }) → Rust command → filesystem/network → result → reactive state → UI
```

### Key Entry Points

1. `src/App.vue` — Orchestrates tabs (Market, Library, IDE, Projects, Settings) and all modals
2. `src/composables/useSkillsManager.ts` — Central orchestrator composing 16+ smaller composables, returns 70+ state properties/methods. Single entry point from App.vue for all business logic
3. `src/composables/useLibraryWorkspace.ts` — Derives UI-ready data for the 3-column Library view
4. `src-tauri/src/lib.rs` — Tauri app init, all 22 registered invoke commands listed here

### Frontend Composable Composition Pattern

`useSkillsManager` composes domain-specific composables via dependency injection:

```typescript
const { ... } = useMarketConfig();
const { ... } = useIdeConfig();
const { ... } = useMarketplaceSearch(marketConfigs, enabledMarkets, marketStatuses);
const { ... } = useLocalInventory(ideOptions, projectPaths, callbacks, t);
const { ... } = useDownloadQueue(callback, t, scanLocalSkills);
// Returns unified export object combining all composable outputs
```

Key patterns used throughout composables:
- **Busy state tracking**: `busy.value` / `busyText.value` refs for async operation UI feedback
- **Callback injection**: Notification callbacks `(msg) => toast.success(msg)` and translate function `t` passed as parameters
- **Project path sync**: `useSkillsManager` maintains a shared `projectPaths` ref synced from `projects` via `watch`

### Frontend ↔ Backend Communication

All Tauri commands use `invoke()` with typed request/response pairs. Most commands wrap parameters in a `request` object:

```typescript
await invoke("clone_local_skill", { request: { skillName, targetPath, ideLabel } });
```

Exception: `search_marketplaces` uses inline parameters.

### Type Mirroring

Types are manually kept in sync between:
- **Rust**: `src-tauri/src/types.rs` — uses `#[serde(rename_all = "camelCase")]` on all structs
- **TypeScript**: `src/composables/types.ts` — matching camelCase interfaces

When adding a new command, update both files. Tauri's invoke layer handles serialization automatically.

### Backend Command Groups (22 commands in lib.rs)

- **Market** (3): `search_marketplaces`, `download_marketplace_skill`, `update_marketplace_skill`
- **Scan** (3): `scan_overview`, `scan_project_ide_dirs`, `scan_project_opencode_skills`
- **Install/Uninstall** (3): `clone_local_skill`, `uninstall_skill`, `adopt_ide_skill`
- **Import/Delete** (2): `import_local_skill`, `delete_local_skills`
- **Versions** (7): `create_skill_version`, `delete_skill_version`, `rename_skill_version`, `set_default_skill_version`, `compare_skill_versions`, `get_skill_package`, `list_skill_packages`
- **Conflict** (2): `analyze_skill_conflict`, `resolve_skill_conflict`
- **Variants** (3): `create_skill_variant`, `update_skill_variant`, `delete_skill_variant`
- **Config** (2): `get_app_config`, `save_app_config`

### i18n

Two locale files (`src/locales/zh-CN.ts`, `en-US.ts`) must stay in sync. Access via dot notation: `t("messages.downloaded", { path })`. Default locale is `zh-CN`. Top-level namespaces: `app`, `sidebar`, `settings`, `market`, `local`, `ide`, `installModal`, `uninstallModal`, `loading`, `messages`, `errors`, `marketSettings`, `download`, `projects`, `library`.

### Key Concepts

- **Skills** are stored centrally at `~/.qing-skill-manager/skills` and **installed** (copied) to IDE-specific directories via `clone_local_skill`
- **Adopt** = copy an unmanaged IDE skill into central storage, then restore local copy at original location
- **Versions** are tracked with metadata in `~/.qing-skill-manager/versions` (uses `.qing-skill-manager-version.json`)
- **Library Workspace** is the primary UI — a 3-column layout (`LibrarySidebar` / `LibraryDetailPanel` / `LibraryVersionRail`)
- **Multi-IDE support**: 12 IDEs. IDE path mappings are in `constants.ts` — use `getProjectIdeRelativeDir(ideLabel)` and `buildProjectCloneTargetPath(projectPath, ideLabel)` for path construction

### Storage Keys

All localStorage keys are prefixed with `qingSkillManager.` — defined in `constants.ts:STORAGE_KEYS`. Key entries: `ideOptions`, `projects`, `marketConfigs`, `lastInstallTargets`.

### Styling

Theme switching via `data-theme` attribute + CSS variables. Library workspace uses dedicated variables (`--library-bg`, `--library-sidebar-bg`, `--library-surface`, `--library-border`, `--library-accent`).
