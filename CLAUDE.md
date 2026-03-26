# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
pnpm install              # Install dependencies
pnpm tauri dev            # Start full dev environment (Vite + Tauri)
pnpm dev                  # Frontend-only dev server (port 1420)

pnpm build                # Type-check + Vite production build
pnpm tauri build          # Build desktop app bundle

pnpm verify               # Full verification: check + test
pnpm check                # Static analysis (frontend typecheck + cargo check/clippy)
pnpm test                 # All tests (frontend logic + typecheck + build verify + cargo test)
pnpm test:frontend:logic  # Run frontend logic tests only
pnpm test:rust            # Run Rust tests only (cd src-tauri && cargo test)
pnpm typecheck            # vue-tsc --noEmit
```

## Architecture

**Tauri 2 desktop app**: Vue 3 + TypeScript frontend, Rust backend. No centralized store — state is managed via Vue 3 composables using the Composition API.

### Frontend (`src/`)

- **App.vue** — Root component, orchestrates tab navigation (Market, Library, IDE, Projects, Settings) and all modals
- **Composables (`src/composables/`)** — All business logic lives here, organized by domain:
  - `useSkillsManager.ts` — Central orchestrator, composes 10+ smaller composables
  - `useInstallActions.ts` / `useUninstallActions.ts` — Install/uninstall workflows
  - `useLocalInventory.ts` — Local skill repo scanning
  - `useMarketplaceSearch.ts` / `useDownloadQueue.ts` — Marketplace operations
  - `useVersionManagement.ts` — Version/conflict/variant logic
  - `useLibraryWorkspace.ts` — Derives UI-ready data for the 3-column Library view
  - `useProjectConfig.ts` / `useProjectScan.ts` — Project-level skill management
  - `useIdeConfig.ts` / `useIdeAdoption.ts` — IDE configuration and adoption
  - `types.ts` — All TypeScript type definitions
  - `constants.ts` — IDE path mappings, storage keys
  - `utils.ts` — Path safety, error handling utilities
- **Components (`src/components/`)** — UI components, notably `library/` for the 3-column Library Workspace
- **i18n (`src/locales/`)** — Chinese (zh-CN) and English (en-US) translations

### Backend (`src-tauri/`)

- **`lib.rs`** — Tauri app setup, plugin registration, command handler registration
- **`commands/skills/`** — Skill management commands (scan, conflict, version, variant, config)
- **`commands/market.rs`** — Marketplace search/download commands
- **`utils/`** — Path normalization, security validation, download helpers
- **`types.rs`** — Backend type definitions (mirrored with frontend `types.ts`)

### Frontend ↔ Backend Communication

All communication uses Tauri's `invoke()` with typed request/response pairs. Key commands: `scan_overview`, `clone_local_skill`, `search_marketplaces`, `download_marketplace_skill`, `analyze_skill_conflict`, `scan_project_ide_dirs`.

### Testing

Frontend logic tests are plain Node.js `.mjs` files (no test framework) in `src/composables/`. Rust tests are inline `#[cfg(test)]` modules. The `pnpm verify` command runs the full verification chain.

### Key Concepts

- **Skills** are stored in a centralized repo at `~/.qing-skill-manager/skills` and installed (cloned/symlinked) to IDE-specific directories
- **Library Workspace** is the primary UI — a 3-column layout (sidebar, detail panel, version rail)
- **Multi-IDE support**: Claude Code, Cursor, Windsurf, AIDE, Cline, Roo Code, Trae, Augment, OpenCode
- **Version/Variant system**: Skills can have multiple versions and variants with conflict detection
