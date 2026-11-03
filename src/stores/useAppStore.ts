import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { tauriApi, DEFAULT_SETTINGS, isTauri } from "../services/api";
import type {
  AppSettings,
  TranscriptionRecord,
  SystemStatus,
  DictationState,
} from "../types";

interface AppState {
  settings: AppSettings;
  systemStatus: SystemStatus;
  history: TranscriptionRecord[];
  configuredKeys: Record<string, boolean>;
  activeTab: "dashboard" | "settings";
  settingsSection: "ai" | "hotkeys" | "general";
  isRecording: boolean;
  isLoading: boolean;
  errorMessage: string | null;

  // Actions
  loadInitialData: () => Promise<void>;
  updateSettings: (partial: Partial<AppSettings>) => Promise<void>;
  saveSettings: () => Promise<void>;
  loadHistory: () => Promise<void>;
  deleteHistoryItem: (id: string) => Promise<void>;
  clearHistory: () => Promise<void>;
  refreshKeyStatuses: () => Promise<void>;
  setActiveTab: (tab: "dashboard" | "settings") => void;
  setSettingsSection: (section: "ai" | "hotkeys" | "general") => void;
  toggleDictation: () => Promise<void>;
  cancelDictation: () => Promise<void>;
  setAppActive: (active: boolean) => Promise<void>;
  setDictationState: (state: DictationState, message?: string) => void;
  setupEventListeners: () => Promise<() => void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  systemStatus: {
    is_active: true,
    display_server: "wayland",
    microphone_ready: false,
    current_dictation_state: "idle",
  },
  history: [],
  configuredKeys: {},
  activeTab: "dashboard",
  settingsSection: "ai",
  isRecording: false,
  isLoading: true,
  errorMessage: null,

  loadInitialData: async () => {
    set({ isLoading: true });
    try {
      const [settings, status, history] = await Promise.all([
        tauriApi.getSettings(),
        tauriApi.getSystemStatus(),
        tauriApi.getHistory(),
      ]);

      set({
        settings,
        systemStatus: status,
        history,
        isLoading: false,
      });

      await get().refreshKeyStatuses();
    } catch (err: unknown) {
      console.error("Failed to load initial data", err);
      set({
        isLoading: false,
        errorMessage: err instanceof Error ? err.message : String(err),
      });
    }
  },

  updateSettings: async (partial: Partial<AppSettings>) => {
    const updated = { ...get().settings, ...partial };
    set({ settings: updated });
    try {
      await tauriApi.saveSettings(updated);
    } catch (err) {
      console.error("Failed to persist settings", err);
    }
  },

  saveSettings: async () => {
    try {
      await tauriApi.saveSettings(get().settings);
    } catch (err) {
      console.error("Failed to save settings", err);
    }
  },

  loadHistory: async () => {
    try {
      const history = await tauriApi.getHistory();
      set({ history });
    } catch (err) {
      console.error("Failed to load history", err);
    }
  },

  deleteHistoryItem: async (id: string) => {
    try {
      await tauriApi.deleteHistoryItem(id);
      set((state) => ({
        history: state.history.filter((h) => h.id !== id),
      }));
    } catch (err) {
      console.error("Failed to delete history item", err);
    }
  },

  clearHistory: async () => {
    try {
      await tauriApi.clearHistory();
      set({ history: [] });
    } catch (err) {
      console.error("Failed to clear history", err);
    }
  },

  refreshKeyStatuses: async () => {
    try {
      const providers = ["groq", "openai", "gemini", "xai"];
      const results: Record<string, boolean> = {};
      for (const p of providers) {
        results[p] = await tauriApi.getApiKeyConfigured(p);
      }
      set({ configuredKeys: results });
    } catch (err) {
      console.error("Failed to check api keys", err);
    }
  },

  setActiveTab: (tab) => set({ activeTab: tab }),
  setSettingsSection: (section) => set({ settingsSection: section }),

  toggleDictation: async () => {
    const { systemStatus } = get();
    try {
      if (systemStatus.current_dictation_state === "listening") {
        await tauriApi.stopDictation();
      } else if (systemStatus.current_dictation_state === "idle" || systemStatus.current_dictation_state === "completed" || systemStatus.current_dictation_state === "error") {
        await tauriApi.startDictation();
      }
    } catch (err: unknown) {
      console.error("Dictation toggle failed", err);
      set((state) => ({
        systemStatus: {
          ...state.systemStatus,
          current_dictation_state: "error",
          last_error: err instanceof Error ? err.message : String(err),
        },
      }));
    }
  },

  cancelDictation: async () => {
    try {
      await tauriApi.cancelDictation();
      set((state) => ({
        systemStatus: {
          ...state.systemStatus,
          current_dictation_state: "idle",
        },
      }));
    } catch (err) {
      console.error("Failed to cancel dictation", err);
    }
  },

  setAppActive: async (active: boolean) => {
    try {
      await tauriApi.setAppActive(active);
      set((state) => ({
        systemStatus: { ...state.systemStatus, is_active: active },
      }));
    } catch (err) {
      console.error("Failed to update active state", err);
    }
  },

  setDictationState: (dictationState: DictationState, message?: string) => {
    set((state) => ({
      systemStatus: {
        ...state.systemStatus,
        current_dictation_state: dictationState,
        state_message: message,
      },
    }));
  },

  setupEventListeners: async () => {
    if (!isTauri()) {
      return () => {};
    }

    const unlistenState = await listen<{
      state: DictationState;
      message?: string;
      inserted_text?: string;
      error?: string;
    }>("dictation-state-changed", (event) => {
      set((state) => ({
        systemStatus: {
          ...state.systemStatus,
          current_dictation_state: event.payload.state,
          state_message: event.payload.message,
          last_inserted_text: event.payload.inserted_text,
          last_error: event.payload.error,
        },
      }));

      if (event.payload.state === "completed") {
        get().loadHistory();
      }
    });

    const unlistenAppStatus = await listen<{ is_active: boolean }>(
      "app-active-changed",
      (event) => {
        set((state) => ({
          systemStatus: {
            ...state.systemStatus,
            is_active: event.payload.is_active,
          },
        }));
      }
    );

    return () => {
      unlistenState();
      unlistenAppStatus();
    };
  },
}));
