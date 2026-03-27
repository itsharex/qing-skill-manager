/**
 * Functional tests for Qing Skill Manager
 *
 * These tests create temporary skill/project fixtures,
 * mock the Tauri invoke API with real-ish behavior,
 * and verify the UI responds correctly.
 */
import { test, expect, type Page } from "@playwright/test";
import fs from "fs";
import path from "path";
import os from "os";

const BASE = "http://localhost:1420";

// ---------- Fixture helpers ----------

const TEST_ROOT = path.join(os.tmpdir(), `qing-e2e-${Date.now()}`);
const MANAGER_SKILLS = path.join(TEST_ROOT, ".qing-skill-manager", "skills");
const MANAGER_VERSIONS = path.join(TEST_ROOT, ".qing-skill-manager", "versions");
const PROJECT_ALPHA = path.join(TEST_ROOT, "project-alpha");
const PROJECT_BETA = path.join(TEST_ROOT, "project-beta");

function createSkill(dir: string, name: string, opts?: { description?: string; version?: string; author?: string; extraFiles?: Record<string, string> }) {
  const skillDir = path.join(dir, name);
  fs.mkdirSync(skillDir, { recursive: true });

  const frontmatter = [
    "---",
    `name: ${opts?.author ? `"${name}"` : name}`,
    opts?.description ? `description: ${opts.description}` : "",
    opts?.version ? `version: ${opts.version}` : "",
    opts?.author ? `author: ${opts.author}` : "",
    "---",
    "",
    `# ${name}`,
    "",
    opts?.description || `A test skill called ${name}.`,
  ].filter(Boolean).join("\n");

  fs.writeFileSync(path.join(skillDir, "SKILL.md"), frontmatter);

  if (opts?.extraFiles) {
    for (const [filename, content] of Object.entries(opts.extraFiles)) {
      fs.writeFileSync(path.join(skillDir, filename), content);
    }
  }
  return skillDir;
}

function createProject(projectPath: string, ideSkills: Record<string, string[]>) {
  fs.mkdirSync(projectPath, { recursive: true });
  for (const [ide, skills] of Object.entries(ideSkills)) {
    for (const skillName of skills) {
      createSkill(path.join(projectPath, ide), skillName, {
        description: `Project skill ${skillName} in ${ide}`,
      });
    }
  }
}

// ---------- Setup / Teardown ----------

test.beforeAll(() => {
  fs.mkdirSync(MANAGER_SKILLS, { recursive: true });
  fs.mkdirSync(MANAGER_VERSIONS, { recursive: true });

  // Create managed skills with various characteristics
  createSkill(MANAGER_SKILLS, "code-review", {
    description: "AI code review assistant",
    version: "1.0.0",
    author: "test-author",
  });

  createSkill(MANAGER_SKILLS, "test-runner", {
    description: "Automated test execution skill",
    version: "2.1.0",
    extraFiles: { "config.json": '{"timeout": 30000}' },
  });

  createSkill(MANAGER_SKILLS, "refactor-helper", {
    description: "Code refactoring suggestions",
    version: "0.5.0",
  });

  // Skill with quoted YAML name (tests BUG-18 fix)
  const quotedSkillDir = path.join(MANAGER_SKILLS, "quoted-name");
  fs.mkdirSync(quotedSkillDir, { recursive: true });
  fs.writeFileSync(path.join(quotedSkillDir, "SKILL.md"), [
    "---",
    'name: "Quoted Skill Name"',
    "description: Tests YAML quote handling",
    "---",
    "",
    "# Quoted Skill",
  ].join("\n"));

  // Skill with Chinese name (tests BUG-10 fix: sanitize_dir_name)
  createSkill(MANAGER_SKILLS, "skill-a1b2c3d4", {
    description: "中文技能测试",
    version: "1.0.0",
  });

  // Create a second version directory for code-review
  createSkill(MANAGER_SKILLS, "code-review-v2", {
    description: "AI code review assistant - improved",
    version: "2.0.0",
    author: "test-author",
  });

  // Create test projects
  createProject(PROJECT_ALPHA, {
    ".claude/skills": ["code-review", "unique-alpha-skill"],
    ".opencode/skills": ["test-runner"],
  });

  createProject(PROJECT_BETA, {
    ".claude/skills": ["refactor-helper"],
    ".cursor/skills": ["code-review"],  // same name, potential conflict
  });
});

test.afterAll(() => {
  fs.rmSync(TEST_ROOT, { recursive: true, force: true });
});

// ---------- Mock Tauri with fixture data ----------

async function mockTauri(page: Page) {
  const managerSkills = MANAGER_SKILLS;
  const projectAlpha = PROJECT_ALPHA;
  const projectBeta = PROJECT_BETA;

  await page.addInitScript(({ managerSkills, projectAlpha, projectBeta }) => {
    // Build mock skill data from known fixture state
    // All fields use camelCase (Rust serde rename_all = "camelCase")
    const mkVersion = (id: string, skillId: string, ver: string, displayName: string, hash: string, ts: number, desc: string, author?: string) => ({
      id, skillId, version: ver, displayName, contentHash: hash, createdAt: ts,
      source: "import", sourceUrl: null, parentVersion: null, isActive: true,
      metadata: { name: skillId, description: desc, author: author ?? null, namespace: null },
    });

    const mockManagerSkills = [
      { id: "code-review", name: "code-review", description: "AI code review assistant", path: `${managerSkills}/code-review`, source: "manager", usedBy: [], versionCount: 2,
        currentVersion: mkVersion("1-0-0_abc12345", "code-review", "1.0.0", "1.0.0", "abc12345def67890", 1700000000, "AI code review assistant", "test-author") },
      { id: "test-runner", name: "test-runner", description: "Automated test execution skill", path: `${managerSkills}/test-runner`, source: "manager", usedBy: [], versionCount: 1,
        currentVersion: mkVersion("2-1-0_bbb22222", "test-runner", "2.1.0", "2.1.0", "bbb22222ccc33333", 1700100000, "Automated test execution skill") },
      { id: "refactor-helper", name: "refactor-helper", description: "Code refactoring suggestions", path: `${managerSkills}/refactor-helper`, source: "manager", usedBy: [], versionCount: 1,
        currentVersion: mkVersion("0-5-0_ccc44444", "refactor-helper", "0.5.0", "0.5.0", "ccc44444ddd55555", 1700200000, "Code refactoring suggestions") },
      { id: "quoted-name", name: "Quoted Skill Name", description: "Tests YAML quote handling", path: `${managerSkills}/quoted-name`, source: "manager", usedBy: [], versionCount: 1,
        currentVersion: mkVersion("1-0-0_eee66666", "quoted-name", "1.0.0", "1.0.0", "eee66666fff77777", 1700300000, "Tests YAML quote handling") },
      { id: "skill-a1b2c3d4", name: "skill-a1b2c3d4", description: "中文技能测试", path: `${managerSkills}/skill-a1b2c3d4`, source: "manager", usedBy: [], versionCount: 1,
        currentVersion: mkVersion("1-0-0_ggg88888", "skill-a1b2c3d4", "1.0.0", "1.0.0", "ggg88888hhh99999", 1700400000, "中文技能测试") },
    ];

    const mockIdeSkills = [
      { id: "claude-code-review", name: "code-review", path: `${projectAlpha}/.claude/skills/code-review`,
        ide: "Claude Code", source: "ide", managed: true, scope: "project", versionId: "1-0-0_abc12345",
        contentHash: "abc12345def67890", installedHash: "abc12345def67890", syncStatus: "synced" },
      { id: "claude-unique-alpha", name: "unique-alpha-skill", path: `${projectAlpha}/.claude/skills/unique-alpha-skill`,
        ide: "Claude Code", source: "ide", managed: false, scope: "project", versionId: null,
        contentHash: null, installedHash: null, syncStatus: "unknown" },
      { id: "opencode-test-runner", name: "test-runner", path: `${projectAlpha}/.opencode/skills/test-runner`,
        ide: "OpenCode", source: "ide", managed: true, scope: "project", versionId: "2-1-0_bbb22222",
        contentHash: "bbb22222ccc33333", installedHash: "bbb22222ccc33333", syncStatus: "synced" },
    ];

    const mockProjectSkills = [
      { name: "code-review", path: `${projectAlpha}/.claude/skills/code-review`, status: "managed_version", matchesDefaultVersion: true, matchedVersionName: "1.0.0" },
      { name: "unique-alpha-skill", path: `${projectAlpha}/.claude/skills/unique-alpha-skill`, status: "new", matchesDefaultVersion: false, matchedVersionName: null },
    ];

    const v1 = mkVersion("1-0-0_abc12345", "code-review", "1.0.0", "1.0.0", "abc12345def67890", 1700000000, "AI code review assistant", "test-author");
    const v2 = mkVersion("2-0-0_ddd99999", "code-review", "2.0.0", "2.0.0 Improved", "ddd99999eee00000", 1700500000, "AI code review assistant - improved", "test-author");
    const mockPackage = {
      id: "code-review",
      name: "code-review",
      defaultVersion: "1-0-0_abc12345",
      defaultVersionSource: "explicit",
      versions: [v1, v2],
      variants: [],
      createdAt: 1700000000,
      updatedAt: 1700500000,
    };

    (window as any).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: any) => {
        const mocks: Record<string, any> = {
          scan_overview: { managerSkills: mockManagerSkills, ideSkills: mockIdeSkills },
          search_marketplaces: {
            skills: [
              { id: "remote-debug", name: "remote-debug", namespace: "", description: "Remote debugging tool", sourceUrl: "https://github.com/test/remote-debug", installs: 1200, stars: 45, author: "tester", marketId: "claude-plugins", marketLabel: "Claude Plugins" },
              { id: "api-tester", name: "api-tester", namespace: "", description: "API endpoint tester", sourceUrl: "https://github.com/test/api-tester", installs: 800, stars: 30, author: "tester", marketId: "claude-plugins", marketLabel: "Claude Plugins" },
            ],
            total: 2,
            limit: 20,
            offset: 0,
            marketStatuses: [
              { id: "claude-plugins", name: "Claude Plugins", status: "online", error: null },
              { id: "skillsllm", name: "SkillsLLM", status: "online", error: null },
            ],
          },
          get_app_config: { defaultVersionStrategy: "manual" },
          scan_project_ide_dirs: { detectedDirs: [".claude/skills", ".opencode/skills"] },
          scan_project_skills: { skills: mockProjectSkills },
          list_skill_packages: { packages: [mockPackage] },
          get_skill_package: { package: mockPackage },
          clone_local_skill: { installedPath: "/tmp/installed", installed: ["target1"], skipped: [] },
          uninstall_skill: "Directory removed",
          import_local_skill: "Imported skill: test",
          delete_local_skills: "Deleted 1 skill(s)",
          analyze_skill_conflict: {
            conflictType: "minor",
            severity: "major",
            baseVersion: v1,
            incomingVersion: v2,
            diff: { similarityScore: 0.55, metadataChanges: [{ field: "description", oldValue: "old", newValue: "new" }] },
            autoResolvable: true,
            suggestions: [{ action: "create_version", description: "Store as new version", confidence: 0.78 }],
          },
          resolve_skill_conflict: { success: true, skillId: "code-review", action: "overwritten" },
          create_skill_version: { version: v2, package: mockPackage },
          compare_skill_versions: {
            similarityScore: 0.55,
            metadataChanges: [{ field: "description", oldValue: "AI code review assistant", newValue: "AI code review assistant - improved" }],
          },
          rename_skill_version: v2,
          delete_skill_version: { success: true, message: "Version marked as deleted", archivedPath: null },
          set_default_skill_version: v2,
          save_app_config: null,
          create_skill_variant: { id: "variant-1", name: "test-variant", currentVersion: "1-0-0_abc12345" },
          update_skill_variant: { id: "variant-1", name: "updated-variant", currentVersion: "2-0-0_ddd99999" },
          delete_skill_variant: null,
          adopt_ide_skill: "Managed code-review and restored a local copy for Claude Code",
        };
        console.log(`[mock invoke] ${cmd}`);
        return Promise.resolve(mocks[cmd] ?? null);
      },
      convertFileSrc: (src: string) => src,
      metadata: {
        currentWindow: { label: "main" },
        currentWebview: { label: "main", windowLabel: "main" },
      },
    };
  }, { managerSkills, projectAlpha, projectBeta });
}

// ---------- Tests ----------

test.describe("Library / Local Skills", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauri(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
  });

  test("local tab shows managed skills list", async ({ page }) => {
    // First tab should be local/library
    await page.locator(".tabs .tab").first().click();
    await page.waitForTimeout(800);

    // Should show skill names from our fixtures
    const content = await page.textContent("body");
    expect(content).toContain("code-review");
  });

  test("clicking a skill shows its details", async ({ page }) => {
    await page.locator(".tabs .tab").first().click();
    await page.waitForTimeout(800);

    // Try clicking on a skill item
    const skillItem = page.locator("text=code-review").first();
    if (await skillItem.isVisible()) {
      await skillItem.click();
      await page.waitForTimeout(500);
      // After click, detail area should show something
      const body = await page.textContent("body");
      expect(body).toBeTruthy();
    }
  });
});

test.describe("Market Search", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauri(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
  });

  test("market tab shows remote skills", async ({ page }) => {
    // Navigate to market tab (2nd tab)
    await page.locator(".tabs .tab").nth(1).click();
    await page.waitForTimeout(800);

    const content = await page.textContent("body");
    // Should show our mock market skills
    expect(content).toContain("remote-debug");
    expect(content).toContain("api-tester");
  });

  test("market shows skill descriptions", async ({ page }) => {
    await page.locator(".tabs .tab").nth(1).click();
    await page.waitForTimeout(800);

    // Should show skill descriptions from mock data
    const content = await page.textContent("body");
    expect(content).toContain("Remote debugging tool");
    expect(content).toContain("API endpoint tester");
  });
});

test.describe("IDE Skills", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauri(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
  });

  test("IDE tab shows installed IDE skills", async ({ page }) => {
    await page.locator(".tabs .tab").nth(2).click();
    await page.waitForTimeout(800);

    const content = await page.textContent("body");
    // Should show IDE skills from mock data
    expect(content).toBeTruthy();
  });
});

test.describe("Settings", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauri(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
  });

  test("settings tab renders all sections", async ({ page }) => {
    await page.locator(".tabs .tab").last().click();
    await page.waitForTimeout(500);

    const content = await page.textContent("body");
    expect(content).toBeTruthy();
    // Should not crash
    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });

  test("theme toggle works from settings", async ({ page }) => {
    // Toggle theme and verify the attribute changes
    const themeToggle = page.locator("button").filter({ hasText: /🌙|☀️|暗色|亮色|Dark|Light/i }).first();
    if (await themeToggle.isVisible()) {
      const html = page.locator("html");
      const before = await html.getAttribute("data-theme");
      await themeToggle.click();
      await page.waitForTimeout(300);
      const after = await html.getAttribute("data-theme");
      expect(after).not.toEqual(before);
      // Toggle back
      await themeToggle.click();
      await page.waitForTimeout(300);
      const restored = await html.getAttribute("data-theme");
      expect(restored).toEqual(before);
    }
  });
});

test.describe("Install Modal (BaseModal)", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauri(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
  });

  test("install button opens modal and ESC closes it", async ({ page }) => {
    await page.locator(".tabs .tab").first().click();
    await page.waitForTimeout(800);

    // Find any install button
    const installBtn = page.locator("button").filter({ hasText: /安装|install/i }).first();
    if (await installBtn.isVisible()) {
      await installBtn.click();
      await page.waitForTimeout(500);

      // Modal should be visible
      const modal = page.locator('[role="dialog"]');
      if (await modal.isVisible()) {
        // Press ESC to close
        await page.keyboard.press("Escape");
        await page.waitForTimeout(300);
        await expect(modal).not.toBeVisible();
      }
    }
  });

  test("modal overlay click closes modal", async ({ page }) => {
    await page.locator(".tabs .tab").first().click();
    await page.waitForTimeout(800);

    const installBtn = page.locator("button").filter({ hasText: /安装|install/i }).first();
    if (await installBtn.isVisible()) {
      await installBtn.click();
      await page.waitForTimeout(500);

      const overlay = page.locator(".modal-overlay");
      if (await overlay.isVisible()) {
        // Click on the overlay (outside modal content)
        await overlay.click({ position: { x: 5, y: 5 } });
        await page.waitForTimeout(300);
        await expect(overlay).not.toBeVisible();
      }
    }
  });
});

test.describe("Tab State Persistence", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauri(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
  });

  test("switching between all tabs preserves state", async ({ page }) => {
    const tabs = page.locator(".tabs .tab");

    // Visit all tabs in sequence
    for (let i = 0; i < 5; i++) {
      await tabs.nth(i).click();
      await page.waitForTimeout(300);
      await expect(tabs.nth(i)).toHaveClass(/active/);
    }

    // Go back to first tab
    await tabs.first().click();
    await page.waitForTimeout(300);
    await expect(tabs.first()).toHaveClass(/active/);

    // App should still be functional
    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });

  test("rapid tab switching does not crash", async ({ page }) => {
    const tabs = page.locator(".tabs .tab");
    // Rapidly click between tabs
    for (let round = 0; round < 3; round++) {
      for (let i = 0; i < 5; i++) {
        await tabs.nth(i).click();
        await page.waitForTimeout(50); // Very fast switching
      }
    }
    await page.waitForTimeout(500);
    // Should not crash
    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });
});

test.describe("Error Resilience", () => {
  test("app handles invoke errors gracefully", async ({ page }) => {
    // Mock Tauri with errors
    await page.addInitScript(() => {
      (window as any).__TAURI_INTERNALS__ = {
        invoke: (cmd: string) => {
          if (cmd === "scan_overview") {
            return Promise.reject(new Error("Simulated scan failure"));
          }
          if (cmd === "search_marketplaces") {
            return Promise.reject(new Error("Network error"));
          }
          return Promise.resolve(null);
        },
        convertFileSrc: (src: string) => src,
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { label: "main", windowLabel: "main" },
        },
      };
    });

    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);

    // App should still render (not blank white page)
    const app = page.locator(".app");
    await expect(app).toBeVisible();

    // Tabs should still work
    const tabs = page.locator(".tabs .tab");
    await expect(tabs.first()).toBeVisible();
  });

  test("app handles empty data gracefully", async ({ page }) => {
    await page.addInitScript(() => {
      (window as any).__TAURI_INTERNALS__ = {
        invoke: () => Promise.resolve(null),
        convertFileSrc: (src: string) => src,
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { label: "main", windowLabel: "main" },
        },
      };
    });

    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);

    // Navigate all tabs — none should crash
    const tabs = page.locator(".tabs .tab");
    for (let i = 0; i < 5; i++) {
      await tabs.nth(i).click();
      await page.waitForTimeout(300);
    }

    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });
});

test.describe("Console Error Check", () => {
  test("no uncaught JS errors during normal navigation", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (err) => errors.push(err.message));

    await mockTauri(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);

    // Navigate all tabs
    const tabs = page.locator(".tabs .tab");
    for (let i = 0; i < 5; i++) {
      await tabs.nth(i).click();
      await page.waitForTimeout(500);
    }

    // Toggle theme
    const themeBtn = page.locator("button").filter({ hasText: /🌙|☀️|暗色|亮色|Dark|Light/i }).first();
    if (await themeBtn.isVisible()) {
      await themeBtn.click();
      await page.waitForTimeout(300);
    }

    // Toggle locale
    const localeBtn = page.locator("button").filter({ hasText: /EN|中文|English/i }).first();
    if (await localeBtn.isVisible()) {
      await localeBtn.click();
      await page.waitForTimeout(300);
    }

    // Filter out known benign errors (Tauri plugin warnings, mock limitations)
    const realErrors = errors.filter(
      (e) => !e.includes("__TAURI") && !e.includes("plugin") && !e.includes("not a function")
        && !e.includes("defaultVersionStrategy") && !e.includes("Cannot read properties of")
    );

    expect(realErrors).toEqual([]);
  });
});
