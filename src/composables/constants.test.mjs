import assert from "node:assert/strict";
import { buildProjectCloneTargetPath, getProjectIdeRelativeDir } from "./constants.ts";

function runConstantsTests() {
  // Test getProjectIdeRelativeDir for OpenCode (currently the only configured IDE)
  assert.equal(getProjectIdeRelativeDir("OpenCode"), ".opencode/skills");
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

  console.log("✓ All constants tests passed (8 assertions)");
}

runConstantsTests();
