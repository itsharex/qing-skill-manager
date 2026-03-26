import assert from "node:assert/strict";
import { buildProjectCloneTargetPath, getProjectIdeRelativeDir } from "./constants.ts";

function runConstantsTests() {
  // Test getProjectIdeRelativeDir for all supported IDEs
  assert.equal(getProjectIdeRelativeDir("OpenCode"), ".opencode/skills");
  assert.equal(getProjectIdeRelativeDir("Claude Code"), ".claude/skills");
  assert.equal(getProjectIdeRelativeDir("Codex"), ".codex/skills");
  assert.equal(getProjectIdeRelativeDir("Cursor"), ".cursor/skills");
  assert.equal(getProjectIdeRelativeDir("OpenClaw"), ".openclaw/skills");
  assert.equal(getProjectIdeRelativeDir("UnknownIDE"), null);
  assert.equal(getProjectIdeRelativeDir(""), null);

  // Test buildProjectCloneTargetPath with various project paths
  assert.equal(
    buildProjectCloneTargetPath("/tmp/demo-project", "OpenCode"),
    "/tmp/demo-project/.opencode/skills"
  );

  assert.equal(
    buildProjectCloneTargetPath("/home/user/projects/my-app", "OpenCode"),
    "/home/user/projects/my-app/.opencode/skills"
  );

  // Test with nested project paths
  assert.equal(
    buildProjectCloneTargetPath("/workspace/company/project-name", "OpenCode"),
    "/workspace/company/project-name/.opencode/skills"
  );

  // Test with unknown IDE
  assert.equal(buildProjectCloneTargetPath("/tmp/demo-project", "UnknownIDE"), null);

  // Test with relative paths (should work as-is since function just joins)
  assert.equal(
    buildProjectCloneTargetPath("./my-project", "OpenCode"),
    "./my-project/.opencode/skills"
  );

  // Test with paths containing spaces
  assert.equal(
    buildProjectCloneTargetPath("/tmp/My Project Name", "OpenCode"),
    "/tmp/My Project Name/.opencode/skills"
  );

  // Test buildProjectCloneTargetPath with other IDEs
  assert.equal(
    buildProjectCloneTargetPath("/tmp/demo", "Claude Code"),
    "/tmp/demo/.claude/skills"
  );
  assert.equal(
    buildProjectCloneTargetPath("/tmp/demo", "Cursor"),
    "/tmp/demo/.cursor/skills"
  );
  assert.equal(
    buildProjectCloneTargetPath("/tmp/demo", "Codex"),
    "/tmp/demo/.codex/skills"
  );
  assert.equal(
    buildProjectCloneTargetPath("/tmp/demo", "OpenClaw"),
    "/tmp/demo/.openclaw/skills"
  );

  console.log("✓ All constants tests passed (18 assertions)");
}

runConstantsTests();
