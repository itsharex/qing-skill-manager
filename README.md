# Qing Skill Manager

[English](README.md) | [中文](README_zh-CN.md)

> A desktop app for discovering, managing, and distributing AI skills across multiple IDEs and projects.

<p align="center">
  <img src="docs/screenshots/en-US/market.jpg" width="720" alt="Marketplace Search" />
</p>

Qing Skill Manager gives you a single place to search public skill marketplaces, store skills in a local repository, install them globally to any supported IDE, and clone versioned copies into individual projects — with conflict detection and resolution built in.

Built with **Tauri 2 + Vue 3 + Rust**. Cross-platform desktop app.

## Screenshots

| Local Skills | Market |
|:---:|:---:|
| ![Local Skills](docs/screenshots/en-US/local.jpg) | ![Market](docs/screenshots/en-US/market.jpg) |

| IDE Browser | Projects |
|:---:|:---:|
| ![IDE Browser](docs/screenshots/en-US/ide.jpg) | ![Projects](docs/screenshots/en-US/project.jpg) |

## Features

### Marketplace Search

Search skills from multiple public registries (Claude Plugins, SkillsLLM, SkillsMP) in one unified interface. Download new skills or update existing ones with a single click.

### Local Repository

All downloaded skills are stored centrally at `~/.qing-skill-manager/skills`. Browse, search, filter, and manage your entire skill collection from the Local Skills tab.

### One-Click IDE Installation

Install any local skill to one or more IDEs at once. Supports global installation (available everywhere) and project-level cloning (scoped to a specific project).

### IDE Browser

Switch between IDEs to see what's installed in each one. Uninstall cleanly, or adopt unmanaged skills (ones you placed manually) into your central repository for proper tracking.

### Project Management

Register your projects, configure which IDEs each project targets, and deploy skills directly. The app detects existing project skills automatically and flags version conflicts with clear resolution options (keep, overwrite, or coexist).

### Version Management

Track multiple versions of each skill with Git-style version history. Compare versions side-by-side, set defaults, create variants for different use cases, and pin specific versions to specific projects.

### Custom IDE Support

Don't see your IDE? Add a custom IDE by specifying its name and skills directory path. Your custom IDE then works identically to the built-in ones.

## Supported IDEs

| IDE | Global Path | Project Path |
|-----|------------|--------------|
| Claude Code | `~/.claude/skills` | `.claude/skills` |
| Codex | `~/.codex/skills` | `.codex/skills` |
| Cursor | `~/.cursor/skills` | `.cursor/skills` |
| OpenClaw | `~/.openclaw/skills` | `.openclaw/skills` |
| OpenCode | `~/.config/opencode/skills` | `.opencode/skills` |

Plus any custom IDE you register.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://rustup.rs/)
- [pnpm](https://pnpm.io/)
- macOS: Xcode Command Line Tools

### Install & Run

```bash
git clone <your-fork-url>
cd skills-manager
pnpm install
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

### macOS Security Note

The app is not yet signed with an Apple Developer certificate. On first launch you may see "App is damaged" or "unidentified developer" warnings. Run this to bypass:

```bash
xattr -dr com.apple.quarantine "/Applications/qing-skill-manager.app"
```

## Quick Workflow

1. **Search** — Go to Market, search for a skill, click Download
2. **Browse** — Switch to Local Skills to see your downloaded collection
3. **Install** — Click "Install to IDE" and pick your target IDEs
4. **Project deploy** — In Projects, add your project, configure IDE targets, then link skills
5. **Stay in sync** — The app detects conflicts when project skills differ from your repo and guides you through resolution

## Data Sources

| Source | URL |
|--------|-----|
| Claude Plugins | `https://claude-plugins.dev/api/skills` |
| SkillsLLM | `https://skillsllm.com/api/skills` |
| SkillsMP | `https://skillsmp.com/api/v1/skills/search` (API key may be required) |

## Tech Stack

- **Desktop**: Tauri 2 (Rust backend, WebView frontend)
- **Frontend**: Vue 3 + TypeScript + Vite
- **Language**: English & Simplified Chinese (vue-i18n)

## Acknowledgement

Qing Skill Manager is built on top of the original [skills-manager](https://github.com/Rito-w/skills-manager). Thanks to the original author and all contributors.

## License

MIT
