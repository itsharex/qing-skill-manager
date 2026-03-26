# Qing Skill Manager

[English](README.md) | [中文](README_zh-CN.md)

**A rebranded fork for multi-IDE AI skill distribution and version management.**

Qing Skill Manager helps you search, import, version, and distribute AI skills across supported IDEs and project workspaces. It provides a unified local repository, clone/copy-based project delivery, project-side conflict detection, and managed version matching for teams or individual creators who want tighter control over their skill assets.

Built with **Tauri 2 + Vue 3 + Rust**, the project targets a practical workflow:

- discover skills from multiple marketplaces
- store them in a managed local repository
- install them into global IDE directories
- clone them into project-specific environments
- track imported versions and resolve project conflicts clearly

## Acknowledgement

Qing Skill Manager is built on top of the original [skills-manager](https://github.com/Rito-w/skills-manager). Thanks to the original author and all contributors for the groundwork that made this fork possible.

> Screenshots are being refreshed for the Qing Skill Manager rebrand.
> Legacy screenshots from the upstream project have been removed to avoid branding confusion.

## ✨ Core Features

- 🧭 **Layout-Adapted Library Workspace**: A new three-column Library view for browsing skills, inspecting versions, and seeing installation context at a glance while preserving the app's original visual language
- 🔍 **Aggregated Market Search**: Search quality skills from public registries in one place
- 📦 **Unified Local Repository**: Centralized management of downloaded skills (`~/.qing-skill-manager/skills`)
- 🚀 **One-Click Installation**: Install unified local skills to target IDEs in seconds
- 🛠️ **Multi-Dimensional Management**: Browse skills per IDE, uninstall cleanly and safely
- ⚙️ **Project Management**: Manage projects, clone skills into projects, and configure IDEs per project

## 🎯 Natively Supported IDEs (Alphabetical Order)

- **Claude Code**: `.claude/skills`
- **Codex**: `.codex/skills`
- **Cursor**: `.cursor/skills`
- **OpenClaw**: `.openclaw/skills`
- **OpenCode**: `.config/opencode/skills` (project: `.opencode/skills`)

## 📖 Usage Guide

### 📥 Installation & Usage

- **General Users**: Build from source or publish releases from your own fork.
- **Developers**: Clone the source code repository to run locally or customize in-depth.

### 🍎 macOS Security Note

Since Apple developer commercial signature is not configured yet, opening the app for the first time may trigger "App is damaged and can't be opened" or "from an unidentified developer" warnings. You can run the following terminal command to bypass it:

```bash
xattr -dr com.apple.quarantine "/Applications/qing-skill-manager.app"
```

### 🔍 1) Market

- Aggregated display of available skills from configured data sources.
- Clicking download automatically adds it to your local repository. If an older version exists, an "Update" button will be highlighted instead.

### 🗂️ 2) Local Skills

- Overview of all skills currently downloaded to your local repository.
- Click "Install" to select target IDEs for deployment.

### 🧱 3) Library Workspace

- The Local tab now uses a **Library workspace** adapted from the layout sketch, with sidebar, detail panel, and version rail.
- The left column supports quick search, multi-select, and skill switching; the center column focuses on description, path, source, installation status, project mappings, and clone-to-project entry points; the right rail surfaces version history and comparison actions.
- This view is now the main frontend surface for version mapping, project usage inspection, and clone-to-project workflows.

### ⌨️ 4) IDE Browser

- Switch workspace perspective (e.g., VSCode or Cursor) to view installed skills for each IDE.
- Safe Uninstallation: Removes the installed skill directory safely.
- Can't find your IDE? Click "Add Custom IDE" in the top right to register its skills directory.

## 🏗 Frontend Architecture Notes

- `src/App.vue` remains the application orchestrator, but the Local tab now mounts `src/components/library/LibraryWorkspace.vue`.
- `src/components/library/` contains the new Library domain UI:
  - `LibrarySidebar.vue`
  - `LibraryDetailPanel.vue`
  - `LibraryVersionRail.vue`
  - `LibraryWorkspace.vue`
- `src/composables/useLibraryWorkspace.ts` derives UI-ready Library data from existing local skills, IDE installations, project snapshots, and version metadata.
- Existing Market / IDE / Projects / Settings flows remain intact; the refactor is intentionally frontend-focused and does not require Rust backend changes.

## 👨‍💻 Installation & Development

### Prerequisites

- Node.js (LTS recommended)
- Rust (installed via rustup)
- macOS: Xcode Command Line Tools

### Local Development

```bash
pnpm install
pnpm tauri dev
```

### Testing

- Test script documentation: [`docs/testing-guide.md`](docs/testing-guide.md)
- Recommended project check:

```bash
pnpm run check:project
```

- Frontend-only verification for the Library workspace refactor:

```bash
pnpm run typecheck
pnpm run build
```

### Build & Release

```bash
pnpm tauri build
```

## 📡 Remote Data Sources

- **Claude Plugins**: `https://claude-plugins.dev/api/skills`
- **SkillsLLM**: `https://skillsllm.com/api/skills`
- **SkillsMP**: `https://skillsmp.com/api/v1/skills/search` (API key configuration may be required due to CORS restrictions)
- Source Code Download Proxy: `https://github-zip-api.val.run/zip?source=<repo>`

## 🛠 Tech Stack

- Desktop Runtime Framework: **Tauri 2**
- Frontend UI Layer: **Vue 3** + **TypeScript** + **Vite**
- System Operations Layer: **Rust** (Command side)

## 📄 License

MIT
