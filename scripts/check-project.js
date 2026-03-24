#!/usr/bin/env node
import { execSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, "..");

const steps = [
  { label: "frontend logic tests", command: 'node "src/composables/constants.test.mjs"' },
  { label: "typecheck", command: "pnpm run typecheck" },
  { label: "production build", command: "pnpm run build:verify" },
  { label: "rust tests", command: "cargo test", cwd: path.join(root, "src-tauri") }
];

for (const step of steps) {
  process.stdout.write(`\n==> ${step.label}\n`);
  execSync(step.command, {
    cwd: step.cwd ?? root,
    stdio: "inherit"
  });
}

process.stdout.write("\nAll project checks passed.\n");
