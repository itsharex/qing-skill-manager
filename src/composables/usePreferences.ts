import { ref, watch, onMounted } from "vue";
import { i18n, supportedLocales, type SupportedLocale } from "../i18n";

const STORAGE_KEYS = {
  LOCALE: "qingSkillManager.locale",
  THEME: "qingSkillManager.theme"
} as const;

type Theme = "light" | "dark";

export function usePreferences() {
  const theme = ref<Theme>("light");
  const locale = ref<SupportedLocale>("zh-CN");

  const applyTheme = (next: Theme) => {
    document.documentElement.setAttribute("data-theme", next);
  };

  const loadLocale = (): SupportedLocale => {
    const stored = localStorage.getItem(STORAGE_KEYS.LOCALE) as SupportedLocale | null;
    if (stored && supportedLocales.includes(stored)) return stored;
    const browser = navigator.language.startsWith("zh") ? "zh-CN" : "en-US";
    return browser as SupportedLocale;
  };

  const loadTheme = (): Theme => {
    const stored = localStorage.getItem(STORAGE_KEYS.THEME);
    if (stored === "dark" || stored === "light") return stored;
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  };

  watch(locale, (next) => {
    i18n.global.locale.value = next;
    localStorage.setItem(STORAGE_KEYS.LOCALE, next);
  });

  watch(theme, (next) => {
    applyTheme(next);
    localStorage.setItem(STORAGE_KEYS.THEME, next);
  });

  const toggleLocale = () => {
    locale.value = locale.value === "zh-CN" ? "en-US" : "zh-CN";
  };

  const toggleTheme = () => {
    theme.value = theme.value === "light" ? "dark" : "light";
  };

  onMounted(() => {
    locale.value = loadLocale();
    theme.value = loadTheme();
    i18n.global.locale.value = locale.value;
    applyTheme(theme.value);
  });

  return {
    theme,
    locale,
    toggleTheme,
    toggleLocale,
    applyTheme,
    loadLocale,
    loadTheme
  };
}
