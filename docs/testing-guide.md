# Testing Guide

This document explains the project's current test and verification scripts, what each one checks, and when to use them.

## Overview

The project currently has three main kinds of validation:

1. **Frontend logic tests**
2. **Frontend static/build verification**
3. **Rust backend unit tests**

These are exposed through npm / pnpm scripts in `package.json`.

---

## Script Reference

### `pnpm run test:frontend:logic`

**Command**

```bash
node "src/composables/constants.test.mjs"
```

**Purpose**

Runs the lightweight frontend logic test file for path and IDE-directory resolution.

**What it validates**

File: `src/composables/constants.test.mjs`

This test currently verifies:

- `getProjectIdeRelativeDir("OpenCode")` returns `.opencode/skills`
- unsupported IDE labels return `null`
- `buildProjectCloneTargetPath(...)` correctly builds clone target paths
- nested project paths work correctly
- relative project paths work correctly
- project paths containing spaces work correctly

**When to run**

- After changing `src/composables/constants.ts`
- After changing clone/copy path construction logic
- Before changing project import / clone target behavior

**Why it matters**

This is the fastest test in the repo and catches path-construction regressions before they affect UI flows.

---

### `pnpm run typecheck`

**Command**

```bash
vue-tsc --noEmit
```

**Purpose**

Runs TypeScript and Vue type checking without generating build output.

**What it validates**

- Vue component props/events/types are consistent
- composable return types are valid
- renamed fields like `installed` / `installTargets` are propagated correctly
- frontend code compiles at the type level

**When to run**

- After changing Vue components
- After changing shared frontend types in `src/composables/types.ts`
- After changing event names or payload shapes

**Why it matters**

It catches frontend integration mistakes that simple logic tests cannot catch.

---

### `pnpm run build:verify`

**Command**

```bash
vite build --mode production
```

**Purpose**

Runs a production frontend build to verify the app still bundles successfully.

**What it validates**

- frontend source can be bundled by Vite
- component/module imports are valid
- no build-breaking syntax or module-resolution errors exist

**When to run**

- After changing frontend structure
- After removing files such as updater-related components/stores
- Before committing UI refactors

**Why it matters**

Typecheck can pass while bundling still fails. This script catches that class of problem.

---

### `pnpm run test:rust`

**Command**

```bash
cd src-tauri && cargo test
```

**Purpose**

Runs the Rust backend unit test suite.

**Current coverage areas**

The Rust tests currently cover logic in:

- `src-tauri/src/commands/skills.rs`
- `src-tauri/src/utils/path.rs`
- `src-tauri/src/utils/security.rs`

Key backend scenarios covered include:

#### Versioning and conflict logic

- stable/default version selection
- explicit default override behavior
- content hash matching
- duplicate vs conflict classification
- project skill version matching

#### Clone/copy workflow

- clone target path resolution
- clone succeeds into a valid target directory
- clone rejects targets outside the allowed home scope

#### Managed skill workflow

- adopt flow copies skill into manager storage
- adopt flow restores a local copy instead of creating links
- managed copy detection in overview scan
- uninstall removes installed directories correctly

#### App configuration

- default config fallback is `manual`
- app config write/read persistence
- invalid default-version strategy is rejected

#### Project scan behavior

- matching managed version is recognized as duplicate/default match
- same-name different-content skill is recognized as conflict

#### Utility functions

- path normalization
- directory name sanitization
- security path validation helpers

**When to run**

- After changing Rust commands
- After changing path security logic
- After changing clone/copy, adopt, uninstall, scan, or version-management behavior

**Why it matters**

This is the main automated verification for backend business logic.

---

### `pnpm run test:frontend`

**Command**

```bash
pnpm run test:frontend:logic && pnpm run typecheck && pnpm run build:verify
```

**Purpose**

Runs the full frontend verification chain.

**What it includes**

1. frontend logic test
2. TypeScript/Vue typecheck
3. production build verification

**When to run**

- After most frontend changes
- Before commit when frontend files were modified

**Why it matters**

This is the best single command for validating frontend changes.

---

### `pnpm run test`

**Command**

```bash
pnpm run test:frontend && pnpm run test:rust
```

**Purpose**

Runs both frontend and backend test layers.

**What it includes**

- frontend logic tests
- frontend typecheck
- frontend production build
- Rust backend tests

**When to run**

- Before major commits
- After cross-stack changes
- When clone/copy model changes affect both frontend and backend

**Why it matters**

This is the main project-wide test command.

---

### `pnpm run check:project`

**Command source**

File: `scripts/check-project.js`

**Purpose**

Provides a single scripted project health check with labeled output.

**Execution order**

1. frontend logic tests
2. typecheck
3. production build
4. rust tests

**Difference from `pnpm run test`**

It covers almost the same validation scope as `pnpm run test`, but:

- runs through a custom Node script
- prints labeled sections
- is easier to extend later with extra checks

**When to run**

- For quick release readiness checks
- Before handing work to another person
- Before commit or PR when you want one standard verification command

**Recommended use**

```bash
pnpm run check:project
```

---

### `pnpm run check`

**Command**

```bash
pnpm run check:frontend && pnpm run check:rust
```

**Purpose**

Runs static verification without the full test chain.

**What it includes**

- `check:frontend` → `pnpm run typecheck`
- `check:rust` → `cargo check && cargo clippy -- -D warnings`

**When to run**

- During development when you want fast static validation
- Before running slower full test flows

**Why it matters**

This is useful for catching compile/lint issues before running all tests.

---

### `pnpm run verify`

**Command**

```bash
pnpm run check && pnpm run test
```

**Purpose**

Runs the strictest built-in full verification chain.

**What it includes**

1. frontend typecheck
2. rust compile check
3. rust clippy with warnings denied
4. frontend logic tests
5. frontend production build
6. rust backend tests

**When to run**

- Before release
- Before creating a PR
- When you want the highest-confidence local verification

**Why it matters**

This is the most complete built-in project verification command.

---

## Recommended Usage

### Fast checks while developing

```bash
pnpm run test:frontend:logic
pnpm run typecheck
cd src-tauri && cargo test
```

### Standard pre-commit check

```bash
pnpm run check:project
```

### Strict final verification

```bash
pnpm run verify
```

---

## Current Test Gaps

Even with the current scripts, there are still areas that are only partially covered:

- real UI end-to-end interaction tests
- modal/dropdown interaction behavior in a browser runtime
- full Tauri desktop integration behavior
- marketplace/network integration behavior under mocked responses

If the project later adds E2E automation, this document should be updated with those new scripts.
