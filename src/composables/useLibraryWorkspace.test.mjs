import assert from "node:assert/strict";

// ============================================================================
// Minimal stubs for Vue reactivity (we only need .value access)
// ============================================================================

function ref(val) {
  return { value: val };
}

function computed(fn) {
  return { get value() { return fn(); } };
}

// Stub vue-i18n translate function
function t(key) {
  return key;
}

// ============================================================================
// Re-implement the pure helper functions from useLibraryWorkspace.ts
// These are extracted here since the composable uses Vue imports.
// ============================================================================

function determineSkillStatus(localSkill, allIdeSkills, currentSkillPackage) {
  const relatedIdeSkills = allIdeSkills.filter(
    (ideSkill) => ideSkill.name === localSkill.name
  );
  const managedIdeSkills = relatedIdeSkills.filter((s) => s.managed);

  if (managedIdeSkills.length === 0) {
    return "not-installed";
  }

  const hasModified = managedIdeSkills.some((s) => s.syncStatus === "modified");
  if (hasModified) {
    return "modified";
  }

  const unmanagedCopies = relatedIdeSkills.filter((s) => !s.managed);
  if (unmanagedCopies.length > 0) {
    return "conflict";
  }

  if (currentSkillPackage && currentSkillPackage.id === localSkill.id) {
    const pkg = currentSkillPackage;
    const defaultVersion = pkg.versions.find((v) => v.id === pkg.defaultVersion);
    if (defaultVersion && localSkill.currentVersion) {
      if (localSkill.currentVersion.id !== defaultVersion.id && !localSkill.currentVersion.isActive) {
        return "outdated";
      }
    }
    if (pkg.versions.length > 1 && defaultVersion) {
      const latestVersion = pkg.versions[0];
      if (latestVersion.id !== defaultVersion.id && latestVersion.source === "market") {
        return "outdated";
      }
    }
  }

  return "installed";
}

function buildIdeInstallations(localSkill, allIdeSkills, allIdeOptions) {
  const installations = [];
  const seen = new Set();

  for (const ideSkill of allIdeSkills) {
    if (ideSkill.name !== localSkill.name) continue;
    if (seen.has(ideSkill.path)) continue;
    seen.add(ideSkill.path);

    const ideOption = allIdeOptions.find((opt) => opt.id === ideSkill.ide);
    installations.push({
      ideId: ideSkill.ide,
      ideLabel: ideOption?.label || ideSkill.ide,
      skillPath: ideSkill.path,
      versionId: ideSkill.versionId,
      isManaged: ideSkill.managed,
      scope: ideSkill.scope,
      syncStatus: ideSkill.syncStatus,
    });
  }

  return installations;
}

function buildProjectMappings(localSkill, allProjects, snapshots) {
  return allProjects.map((project) => {
    const projectSkills = snapshots[project.id] || [];
    const matchingSkill = projectSkills.find((ps) => ps.name === localSkill.name);

    if (!matchingSkill) {
      return {
        projectId: project.id,
        projectName: project.name,
        projectPath: project.path,
        versionId: null,
        versionName: null,
        isDefaultVersion: false,
        ideTargets: project.ideTargets,
        status: "missing",
      };
    }

    const isDefaultVersion = matchingSkill.matchesDefaultVersion ?? false;
    const status =
      matchingSkill.status === "conflict" ? "conflict" :
      matchingSkill.status === "duplicate" || matchingSkill.status === "managed_version" ? "synced" :
      "modified";

    return {
      projectId: project.id,
      projectName: project.name,
      projectPath: project.path,
      versionId: matchingSkill.matchedVersionId || matchingSkill.currentVersion?.id || null,
      versionName: matchingSkill.matchedVersionName || matchingSkill.currentVersion?.displayName || null,
      isDefaultVersion,
      ideTargets: project.ideTargets,
      status,
    };
  });
}

function buildUnmanagedSkills(ideSkills, repoSkills, projects) {
  const managedNames = new Set(repoSkills.map((s) => s.name.toLowerCase()));
  for (const s of repoSkills) {
    const dirName = s.path.split("/").pop();
    if (dirName) managedNames.add(dirName.toLowerCase());
  }
  const unmanagedMap = new Map();

  function getSourceLabel(ideSkill) {
    if (ideSkill.scope === "global") return `${ideSkill.ide} · global`;
    const project = projects.find((p) => ideSkill.path.startsWith(p.path));
    return project ? project.name : ideSkill.ide;
  }

  for (const ideSkill of ideSkills) {
    const ideSkillDirName = ideSkill.path.split("/").pop()?.toLowerCase() || "";
    if (ideSkill.managed || managedNames.has(ideSkill.name.toLowerCase()) || managedNames.has(ideSkillDirName) || ideSkill.scope === "plugin") {
      continue;
    }
    const scope = ideSkill.scope;
    const sourceEntry = {
      ide: ideSkill.ide,
      scope,
      path: ideSkill.path,
      label: getSourceLabel(ideSkill),
      contentHash: ideSkill.contentHash,
    };

    const existing = unmanagedMap.get(ideSkill.name);
    if (existing) {
      existing.unmanagedSources.push(sourceEntry);
    } else {
      unmanagedMap.set(ideSkill.name, {
        id: `unmanaged_${ideSkill.id}`,
        name: ideSkill.name,
        unmanagedSources: [sourceEntry],
      });
    }
  }

  return [...unmanagedMap.values()];
}

// ============================================================================
// Tests
// ============================================================================

let assertions = 0;

function assertEq(actual, expected, msg) {
  assert.deepStrictEqual(actual, expected, msg);
  assertions++;
}

function assertTrue(val, msg) {
  assert.ok(val, msg);
  assertions++;
}

// --- P4: determineSkillStatus ---

function testStatusNotInstalled() {
  const skill = { name: "test", id: "test" };
  const ideSkills = []; // no IDE skills at all
  assertEq(determineSkillStatus(skill, ideSkills, null), "not-installed");
}

function testStatusInstalled() {
  const skill = { name: "test", id: "test" };
  const ideSkills = [
    { name: "test", managed: true, syncStatus: "synced" },
  ];
  assertEq(determineSkillStatus(skill, ideSkills, null), "installed");
}

function testStatusModified() {
  const skill = { name: "test", id: "test" };
  const ideSkills = [
    { name: "test", managed: true, syncStatus: "modified" },
  ];
  assertEq(determineSkillStatus(skill, ideSkills, null), "modified");
}

function testStatusConflict() {
  const skill = { name: "test", id: "test" };
  const ideSkills = [
    { name: "test", managed: true, syncStatus: "synced" },
    { name: "test", managed: false, syncStatus: "unknown" }, // unmanaged copy
  ];
  assertEq(determineSkillStatus(skill, ideSkills, null), "conflict");
}

function testStatusOutdatedInactiveVersion() {
  const skill = {
    name: "test",
    id: "pkg1",
    currentVersion: { id: "old_v", isActive: false },
  };
  const ideSkills = [{ name: "test", managed: true, syncStatus: "synced" }];
  const pkg = {
    id: "pkg1",
    defaultVersion: "new_v",
    versions: [
      { id: "new_v", source: "migration" },
      { id: "old_v", source: "migration" },
    ],
  };
  assertEq(determineSkillStatus(skill, ideSkills, pkg), "outdated");
}

function testStatusOutdatedMarketUpdate() {
  const skill = {
    name: "test",
    id: "pkg1",
    currentVersion: { id: "v1", isActive: true },
  };
  const ideSkills = [{ name: "test", managed: true, syncStatus: "synced" }];
  const pkg = {
    id: "pkg1",
    defaultVersion: "v1",
    versions: [
      { id: "v2", source: "market" }, // latest from market
      { id: "v1", source: "migration" },
    ],
  };
  assertEq(determineSkillStatus(skill, ideSkills, pkg), "outdated");
}

// --- P4: buildIdeInstallations ---

function testBuildInstallations() {
  const skill = { name: "test" };
  const ideSkills = [
    { name: "test", ide: "Claude Code", path: "/a/b/test", managed: true, scope: "global", syncStatus: "synced", versionId: "v1" },
    { name: "test", ide: "OpenCode", path: "/c/d/test", managed: true, scope: "project", syncStatus: "modified", versionId: "v2" },
    { name: "other", ide: "Cursor", path: "/e/f/other", managed: false, scope: "global", syncStatus: "unknown", versionId: null },
  ];
  const ideOptions = [
    { id: "Claude Code", label: "Claude Code" },
    { id: "OpenCode", label: "OpenCode" },
  ];

  const result = buildIdeInstallations(skill, ideSkills, ideOptions);
  assertEq(result.length, 2);
  assertEq(result[0].ideId, "Claude Code");
  assertEq(result[0].syncStatus, "synced");
  assertEq(result[1].ideId, "OpenCode");
  assertEq(result[1].scope, "project");
}

function testBuildInstallationsDedup() {
  const skill = { name: "test" };
  const ideSkills = [
    { name: "test", ide: "Claude Code", path: "/same/path", managed: true, scope: "global", syncStatus: "synced", versionId: "v1" },
    { name: "test", ide: "Claude Code", path: "/same/path", managed: true, scope: "global", syncStatus: "synced", versionId: "v1" },
  ];

  const result = buildIdeInstallations(skill, ideSkills, []);
  assertEq(result.length, 1);
}

// --- P4: buildProjectMappings ---

function testBuildProjectMappingsMissing() {
  const skill = { name: "test" };
  const projects = [{ id: "p1", name: "Project 1", path: "/proj", ideTargets: ["OpenCode"] }];
  const snapshots = {};

  const result = buildProjectMappings(skill, projects, snapshots);
  assertEq(result.length, 1);
  assertEq(result[0].status, "missing");
  assertEq(result[0].versionId, null);
}

function testBuildProjectMappingsDuplicate() {
  const skill = { name: "test" };
  const projects = [{ id: "p1", name: "Project 1", path: "/proj", ideTargets: ["OpenCode"] }];
  const snapshots = {
    p1: [{
      name: "test",
      status: "duplicate",
      matchesDefaultVersion: true,
      matchedVersionId: "v1",
      matchedVersionName: "1.0.0",
    }],
  };

  const result = buildProjectMappings(skill, projects, snapshots);
  assertEq(result[0].status, "synced");
  assertEq(result[0].versionId, "v1");
  assertTrue(result[0].isDefaultVersion);
}

function testBuildProjectMappingsManagedVersion() {
  const skill = { name: "test" };
  const projects = [{ id: "p1", name: "Project 1", path: "/proj", ideTargets: [] }];
  const snapshots = {
    p1: [{
      name: "test",
      status: "managed_version",
      matchesDefaultVersion: false,
      matchedVersionId: "v2",
      matchedVersionName: "2.0.0",
    }],
  };

  const result = buildProjectMappings(skill, projects, snapshots);
  assertEq(result[0].status, "synced");
  assertEq(result[0].versionId, "v2");
  assertEq(result[0].isDefaultVersion, false);
}

function testBuildProjectMappingsConflict() {
  const skill = { name: "test" };
  const projects = [{ id: "p1", name: "Project 1", path: "/proj", ideTargets: [] }];
  const snapshots = {
    p1: [{
      name: "test",
      status: "conflict",
      matchesDefaultVersion: null,
      matchedVersionId: null,
      matchedVersionName: null,
      currentVersion: { id: "incoming_v", displayName: "Incoming" },
    }],
  };

  const result = buildProjectMappings(skill, projects, snapshots);
  assertEq(result[0].status, "conflict");
  assertEq(result[0].versionId, "incoming_v");
}

// --- P4: Unmanaged skill deduplication ---

function testUnmanagedSkillDedup() {
  const repoSkills = [
    { name: "Managed Skill", path: "/repo/managed-skill" },
  ];
  const ideSkills = [
    // This should be excluded (name matches repo skill)
    { id: "1", name: "Managed Skill", path: "/ide/managed-skill", managed: false, scope: "global", contentHash: "h1" },
    // This is genuinely unmanaged
    { id: "2", name: "New Skill", path: "/ide/claude/new-skill", managed: false, scope: "global", ide: "Claude Code", contentHash: "h2" },
    // Same name in different IDE → should merge into one entry
    { id: "3", name: "New Skill", path: "/ide/opencode/new-skill", managed: false, scope: "global", ide: "OpenCode", contentHash: "h2" },
  ];

  const result = buildUnmanagedSkills(ideSkills, repoSkills, []);
  assertEq(result.length, 1);
  assertEq(result[0].name, "New Skill");
  assertEq(result[0].unmanagedSources.length, 2);
  assertEq(result[0].unmanagedSources[0].ide, "Claude Code");
  assertEq(result[0].unmanagedSources[1].ide, "OpenCode");
}

function testUnmanagedSkillExcludesByDirName() {
  const repoSkills = [
    { name: "Different Display Name", path: "/repo/actual-dir-name" },
  ];
  const ideSkills = [
    // Directory name matches repo skill path's dir name → excluded
    { id: "1", name: "Other Name", path: "/ide/actual-dir-name", managed: false, scope: "global", ide: "Claude Code", contentHash: "h1" },
  ];

  const result = buildUnmanagedSkills(ideSkills, repoSkills, []);
  assertEq(result.length, 0);
}

function testUnmanagedSkillExcludesPlugins() {
  const repoSkills = [];
  const ideSkills = [
    { id: "1", name: "Plugin Skill", path: "/plugins/my-plugin/skills/x", managed: false, scope: "plugin", ide: "Claude Code", contentHash: "h1" },
  ];

  const result = buildUnmanagedSkills(ideSkills, repoSkills, []);
  assertEq(result.length, 0);
}

function testUnmanagedSkillExcludesManagedFlag() {
  const repoSkills = [];
  const ideSkills = [
    { id: "1", name: "Already Managed", path: "/ide/already-managed", managed: true, scope: "global", ide: "Claude Code", contentHash: "h1" },
  ];

  const result = buildUnmanagedSkills(ideSkills, repoSkills, []);
  assertEq(result.length, 0);
}

function testUnmanagedSkillCaseInsensitive() {
  const repoSkills = [
    { name: "My Skill", path: "/repo/my-skill" },
  ];
  const ideSkills = [
    { id: "1", name: "my skill", path: "/ide/something-else", managed: false, scope: "global", ide: "Claude Code", contentHash: "h1" },
  ];

  const result = buildUnmanagedSkills(ideSkills, repoSkills, []);
  assertEq(result.length, 0);
}

// ============================================================================
// Run all tests
// ============================================================================

testStatusNotInstalled();
testStatusInstalled();
testStatusModified();
testStatusConflict();
testStatusOutdatedInactiveVersion();
testStatusOutdatedMarketUpdate();
testBuildInstallations();
testBuildInstallationsDedup();
testBuildProjectMappingsMissing();
testBuildProjectMappingsDuplicate();
testBuildProjectMappingsManagedVersion();
testBuildProjectMappingsConflict();
testUnmanagedSkillDedup();
testUnmanagedSkillExcludesByDirName();
testUnmanagedSkillExcludesPlugins();
testUnmanagedSkillExcludesManagedFlag();
testUnmanagedSkillCaseInsensitive();

console.log(`✓ All useLibraryWorkspace tests passed (${assertions} assertions)`);
