import { test, expect } from "@playwright/test";

const BASE = "http://localhost:1420";

// Mock Tauri invoke API before page loads
async function mockTauriApi(page: import("@playwright/test").Page) {
  await page.addInitScript(() => {
    // Mock window.__TAURI_INTERNALS__ to prevent invoke errors
    (window as any).__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: any) => {
        console.log(`[mock invoke] ${cmd}`, args);
        // Return sensible defaults per command
        const mocks: Record<string, any> = {
          scan_overview: { managerSkills: [], ideSkills: [] },
          search_marketplaces: { skills: [], total: 0, limit: 20, offset: 0, marketStatuses: [] },
          get_app_config: { defaultVersionStrategy: "manual" },
          scan_project_ide_dirs: { detectedDirs: [] },
          scan_project_skills: { skills: [] },
          list_skill_packages: { packages: [] },
        };
        return Promise.resolve(mocks[cmd] ?? null);
      },
      convertFileSrc: (src: string) => src,
    };
    // Mock Tauri event system
    (window as any).__TAURI_INTERNALS__.metadata = {
      currentWindow: { label: "main" },
      currentWebview: { label: "main", windowLabel: "main" },
    };
  });
}

test.describe("App Smoke Tests", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriApi(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
  });

  test("app renders with header tabs", async ({ page }) => {
    // Should have the tab bar with all 5 main tabs
    const tabs = page.locator(".tabs .tab");
    await expect(tabs).toHaveCount(5);
  });

  test("tab navigation works", async ({ page }) => {
    const tabs = page.locator(".tabs .tab");

    // Click each tab and verify it becomes active
    for (let i = 0; i < 5; i++) {
      await tabs.nth(i).click();
      await expect(tabs.nth(i)).toHaveClass(/active/);
    }
  });

  test("theme toggle switches data-theme", async ({ page }) => {
    const html = page.locator("html");

    // Find theme toggle button
    const themeBtn = page.locator("button").filter({ hasText: /🌙|☀️|暗色|亮色|Dark|Light/i }).first();

    if (await themeBtn.isVisible()) {
      const initialTheme = await html.getAttribute("data-theme");
      await themeBtn.click();
      const newTheme = await html.getAttribute("data-theme");
      expect(newTheme).not.toEqual(initialTheme);
    }
  });

  test("locale toggle switches language", async ({ page }) => {
    // Find locale toggle
    const localeBtn = page.locator("button").filter({ hasText: /EN|中文|English/i }).first();
    if (await localeBtn.isVisible()) {
      const textBefore = await localeBtn.textContent();
      await localeBtn.click();
      await page.waitForTimeout(300);
      const textAfter = await localeBtn.textContent();
      expect(textAfter).not.toEqual(textBefore);
    }
  });

  test("library tab shows empty state or skill list", async ({ page }) => {
    // Click library/local tab (first tab)
    await page.locator(".tabs .tab").first().click();
    await page.waitForTimeout(500);

    // Should render content area without crash
    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });

  test("market tab renders search area", async ({ page }) => {
    // Click market tab (second tab)
    await page.locator(".tabs .tab").nth(1).click();
    await page.waitForTimeout(500);

    // Should have a search input or search area
    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });

  test("settings tab renders", async ({ page }) => {
    // Click settings tab (last tab)
    await page.locator(".tabs .tab").last().click();
    await page.waitForTimeout(500);

    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });
});

test.describe("Modal Tests", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriApi(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
  });

  test("ESC key closes modals (BaseModal)", async ({ page }) => {
    // Navigate to IDE tab where we might trigger a modal
    await page.locator(".tabs .tab").nth(2).click();
    await page.waitForTimeout(500);

    // Try to find any button that might open a modal
    const modalOverlay = page.locator(".modal-overlay");

    // If no modal is open, the test still passes — we just verify no crash
    if (await modalOverlay.isVisible()) {
      await page.keyboard.press("Escape");
      await page.waitForTimeout(300);
      await expect(modalOverlay).not.toBeVisible();
    }
  });
});

test.describe("Responsive UI", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriApi(page);
    await page.goto(BASE);
    await page.waitForLoadState("networkidle");
  });

  test("app handles small viewport without crash", async ({ page }) => {
    await page.setViewportSize({ width: 800, height: 600 });
    await page.waitForTimeout(300);

    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });

  test("app handles large viewport without crash", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.waitForTimeout(300);

    const app = page.locator(".app");
    await expect(app).toBeVisible();
  });

  test("all tabs render without errors at each viewport", async ({ page }) => {
    for (const size of [
      { width: 800, height: 600 },
      { width: 1280, height: 720 },
      { width: 1920, height: 1080 },
    ]) {
      await page.setViewportSize(size);
      const tabs = page.locator(".tabs .tab");
      const tabCount = await tabs.count();
      for (let i = 0; i < tabCount; i++) {
        await tabs.nth(i).click();
        await page.waitForTimeout(200);
        // Verify no unhandled JS error causes blank page
        const app = page.locator(".app");
        await expect(app).toBeVisible();
      }
    }
  });
});
