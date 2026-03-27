<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { i18n, supportedLocales, type SupportedLocale } from "../i18n";
import { useToast } from "../composables/useToast";

const { t } = useI18n();
const toast = useToast();

type ThemeMode = "light" | "dark" | "system";
type DefaultVersionStrategy = "manual" | "latest" | "stable";
const themeKey = "qingSkillManager.theme";
const localeKey = "qingSkillManager.locale";
const theme = ref<ThemeMode>("system");
const locale = ref<SupportedLocale>("zh-CN");
const defaultVersionStrategy = ref<DefaultVersionStrategy>("manual");
const defaultVersionStrategyLoaded = ref(false);
const savingDefaultVersionStrategy = ref(false);
const defaultVersionStrategyStatus = ref<"idle" | "saved" | "error">("idle");


// Apply theme to document
const applyTheme = (mode: ThemeMode) => {
  let effectiveTheme: "light" | "dark";
  if (mode === "system") {
    effectiveTheme = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } else {
    effectiveTheme = mode;
  }
  document.documentElement.setAttribute("data-theme", effectiveTheme);
};

// Load saved preferences
const loadTheme = (): ThemeMode => {
  const stored = localStorage.getItem(themeKey);
  if (stored === "dark" || stored === "light" || stored === "system") {
    return stored;
  }
  return "system";
};

const loadLocale = (): SupportedLocale => {
  const stored = localStorage.getItem(localeKey) as SupportedLocale | null;
  if (stored && supportedLocales.includes(stored)) return stored;
  const browser = navigator.language.startsWith("zh") ? "zh-CN" : "en-US";
  return browser as SupportedLocale;
};

async function loadDefaultVersionStrategy(): Promise<DefaultVersionStrategy> {
  const response = await invoke("get_app_config") as { config: { defaultVersionStrategy: DefaultVersionStrategy } };
  return response.config.defaultVersionStrategy;
}

watch(theme, (next) => {
  applyTheme(next);
  localStorage.setItem(themeKey, next);
});

watch(locale, (next) => {
  i18n.global.locale.value = next;
  localStorage.setItem(localeKey, next);
});

watch(defaultVersionStrategy, async (next, previous) => {
  if (!defaultVersionStrategyLoaded.value || next === previous) return;
  savingDefaultVersionStrategy.value = true;
  defaultVersionStrategyStatus.value = "idle";
  try {
    await invoke("save_app_config", {
      request: { defaultVersionStrategy: next }
    });
    defaultVersionStrategyStatus.value = "saved";
    toast.success(t("settings.versionDefaults.saved"));
  } catch (error) {
    defaultVersionStrategyStatus.value = "error";
    toast.error(String(error));
  } finally {
    savingDefaultVersionStrategy.value = false;
  }
});

onMounted(async () => {
  theme.value = loadTheme();
  locale.value = loadLocale();
  defaultVersionStrategy.value = await loadDefaultVersionStrategy();
  defaultVersionStrategyLoaded.value = true;
  i18n.global.locale.value = locale.value;
  applyTheme(theme.value);

  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", () => {
      if (theme.value === "system") {
        applyTheme("system");
      }
    });
});
</script>

<template>
  <div class="settings-panel">
    <!-- About Section -->
    <section class="settings-section">
      <h2 class="section-title">{{ t("settings.about.title") }}</h2>
      <div class="about-content">
        <div class="app-info">
          <span class="app-name">Qing Skill Manager</span>
          <span class="version-badge">v0.3.25</span>
        </div>

        <!-- Sponsor -->
        <div class="sponsor-section">
          <span class="sponsor-label">{{ t("settings.about.sponsor") }}</span>
          <div class="sponsor-logo-wrap">
            <img src="../assets/sponsor-logo.svg" alt="Sponsor" class="sponsor-logo" />
          </div>
        </div>
      </div>
    </section>

    <!-- Appearance Section -->
    <section class="settings-section">
      <h2 class="section-title">{{ t("settings.appearance.title") }}</h2>
      <div class="appearance-content">
        <!-- Theme -->
        <div class="setting-row">
          <label class="setting-label">{{ t("settings.appearance.theme") }}</label>
          <div class="theme-options">
            <button
              class="theme-btn"
              :class="{ active: theme === 'light' }"
              @click="theme = 'light'"
            >
              <svg class="theme-icon" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M12 4a1 1 0 011 1v1a1 1 0 11-2 0V5a1 1 0 011-1Zm6.36 2.64a1 1 0 010 1.41l-.7.7a1 1 0 11-1.41-1.41l.7-.7a1 1 0 011.41 0ZM20 11a1 1 0 010 2h-1a1 1 0 110-2h1Zm-8 2a3 3 0 100-6 3 3 0 000 6Zm-7 0a1 1 0 010-2H4a1 1 0 110-2h1a1 1 0 110 2H4a1 1 0 010 2Zm1.64-7.95a1 1 0 011.41 0l.7.7a1 1 0 11-1.41 1.41l-.7-.7a1 1 0 010-1.41ZM12 18a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1Zm7.07-1.07a1 1 0 010 1.41l-.7.7a1 1 0 11-1.41-1.41l.7-.7a1 1 0 011.41 0ZM6.34 16.93a1 1 0 011.41 0l.7.7a1 1 0 11-1.41 1.41l-.7-.7a1 1 0 010-1.41Z"
                />
              </svg>
              <span>{{ t("settings.appearance.light") }}</span>
            </button>
            <button
              class="theme-btn"
              :class="{ active: theme === 'dark' }"
              @click="theme = 'dark'"
            >
              <svg class="theme-icon" viewBox="0 0 24 24" fill="currentColor">
                <path
                  d="M21 14.5A8.5 8.5 0 019.5 3a.9.9 0 00-.9.9 9.6 9.6 0 0010.5 10.5.9.9 0 00.9-.9Z"
                />
              </svg>
              <span>{{ t("settings.appearance.dark") }}</span>
            </button>
            <button
              class="theme-btn"
              :class="{ active: theme === 'system' }"
              @click="theme = 'system'"
            >
              <svg class="theme-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="3" width="20" height="14" rx="2" />
                <path d="M8 21h8M12 17v4" />
              </svg>
              <span>{{ t("settings.appearance.system") }}</span>
            </button>
          </div>
        </div>

        <!-- Language -->
        <div class="setting-row">
          <label class="setting-label">{{ t("settings.appearance.language") }}</label>
          <div class="language-options">
            <button
              class="lang-btn"
              :class="{ active: locale === 'zh-CN' }"
              @click="locale = 'zh-CN'"
            >
              中文
            </button>
            <button
              class="lang-btn"
              :class="{ active: locale === 'en-US' }"
              @click="locale = 'en-US'"
            >
              English
            </button>
          </div>
        </div>

        <div class="setting-row">
          <label class="setting-label">{{ t("settings.versionDefaults.title") }}</label>
          <div class="strategy-options">
            <button
              class="theme-btn"
              :class="{ active: defaultVersionStrategy === 'manual' }"
              :disabled="savingDefaultVersionStrategy"
              @click="defaultVersionStrategy = 'manual'"
            >
              <span>{{ t("settings.versionDefaults.manual") }}</span>
            </button>
            <button
              class="theme-btn"
              :class="{ active: defaultVersionStrategy === 'latest' }"
              :disabled="savingDefaultVersionStrategy"
              @click="defaultVersionStrategy = 'latest'"
            >
              <span>{{ t("settings.versionDefaults.latest") }}</span>
            </button>
            <button
              class="theme-btn"
              :class="{ active: defaultVersionStrategy === 'stable' }"
              :disabled="savingDefaultVersionStrategy"
              @click="defaultVersionStrategy = 'stable'"
            >
              <span>{{ t("settings.versionDefaults.stable") }}</span>
            </button>
          </div>
          <p class="setting-note">{{ t("settings.versionDefaults.description") }}</p>
          <p class="setting-note status-line">
            <span v-if="savingDefaultVersionStrategy">{{ t("settings.versionDefaults.saving") }}</span>
            <span v-else-if="defaultVersionStrategyStatus === 'saved'">{{ t("settings.versionDefaults.saved") }}</span>
            <span v-else-if="defaultVersionStrategyStatus === 'error'">{{ t("settings.versionDefaults.saveFailed") }}</span>
          </p>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 20px;
  flex: 1;
  overflow: auto;
}

.settings-section {
  background: var(--color-panel-bg);
  border: 1px solid var(--color-panel-border);
  border-radius: 14px;
  padding: 20px;
  box-shadow: 0 4px 16px var(--color-panel-shadow);
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  margin: 0 0 16px;
  color: var(--color-text);
}

/* About Section */
.about-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.app-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-name {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.version-badge {
  font-size: 13px;
  font-weight: 500;
  padding: 4px 10px;
  border-radius: 999px;
  background: var(--color-chip-bg);
  border: 1px solid var(--color-chip-border);
  color: var(--color-muted);
}

.sponsor-section {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
  padding-top: 12px;
  border-top: 1px solid var(--color-panel-border);
}

.sponsor-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.sponsor-logo-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
}

.sponsor-logo {
  width: 80px;
  height: auto;
  opacity: 0.85;
  transition: opacity 0.2s ease;
}

.sponsor-logo:hover {
  opacity: 1;
}

/* Appearance Section */
.appearance-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.setting-row {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-muted);
}

.theme-options {
  display: flex;
  gap: 10px;
}

.theme-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: 10px;
  border: 1px solid var(--color-input-border);
  background: var(--color-input-bg);
  color: var(--color-tab-text);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.theme-btn:hover {
  border-color: var(--color-input-focus);
}

.theme-btn.active {
  background: var(--color-tab-active-bg);
  color: var(--color-tab-active-text);
  border-color: transparent;
}

.theme-icon {
  width: 16px;
  height: 16px;
}

.language-options {
  display: flex;
  gap: 10px;
}

.strategy-options {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.setting-note {
  margin: 0;
  font-size: 12px;
  color: var(--color-muted);
}

.status-line {
  min-height: 18px;
}

.lang-btn {
  padding: 10px 20px;
  border-radius: 10px;
  border: 1px solid var(--color-input-border);
  background: var(--color-input-bg);
  color: var(--color-tab-text);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.lang-btn:hover {
  border-color: var(--color-input-focus);
}

.lang-btn.active {
  background: var(--color-tab-active-bg);
  color: var(--color-tab-active-text);
  border-color: transparent;
}

/* Button styles */
.primary {
  border: none;
  border-radius: 10px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  background: var(--color-primary-bg);
  color: var(--color-primary-text);
  transition: all 0.2s ease;
}

.primary:hover:not(:disabled) {
  transform: translateY(-1px);
  filter: brightness(1.1);
}

.primary:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.ghost {
  background: transparent;
  border: 1px solid var(--color-ghost-border);
  color: var(--color-ghost-text);
  border-radius: 10px;
  padding: 8px 14px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.ghost:hover:not(:disabled) {
  transform: translateY(-1px);
  background: var(--color-card-bg);
}

.ghost:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

/* Responsive */
@media (max-width: 520px) {
  .theme-options {
    flex-wrap: wrap;
  }

  .version-row {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

}
</style>
