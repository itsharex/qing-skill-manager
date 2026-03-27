/**
 * Shared application context object passed to sub-composables,
 * replacing individually-injected callback parameters (toast, t, scanLocalSkills).
 */

export interface AppContext {
  toast: {
    success: (message: string) => void;
    error: (message: string) => void;
  };
  t: (key: string, values?: Record<string, string | number>) => string;
  scanLocalSkills: () => Promise<boolean>;
}
