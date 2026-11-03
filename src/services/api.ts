import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  TranscriptionRecord,
  SystemStatus,
  ProviderValidationResult,
  TranscriptionProviderId,
  RefinementProviderId,
} from "../types";

export const DEFAULT_SETTINGS: AppSettings = {
  is_first_launch: true,
  transcription_provider: "groq",
  transcription_model: "whisper-large-v3-turbo",
  refinement_provider: "gemini",
  refinement_model: "gemini-2.5-flash",
  refinement_mode: "standard",
  enable_refinement: true,
  enable_history: true,
  audio_language: "auto",
  hotkey_toggle: "Control+Alt+Space",
  hotkey_start_app: "Control+Alt+S",
  hotkey_stop_app: "Control+Alt+X",
  launch_at_startup: false,
  start_minimized: false,
  background_mode: true,
  clipboard_fallback_mode: "auto",
  enable_notifications: true,
  microphone_device_id: "default",
};

export const PROVIDER_MODELS: Record<TranscriptionProviderId, string[]> = {
  groq: ["whisper-large-v3-turbo", "whisper-large-v3"],
  openai: ["whisper-1"],
};

export const REFINEMENT_MODELS: Record<RefinementProviderId, string[]> = {
  gemini: ["gemini-2.5-flash", "gemini-2.0-flash", "gemini-1.5-flash"],
  groq: ["openai/gpt-oss-120b", "openai/gpt-oss-20b", "llama-3.3-70b-versatile", "llama-3.1-8b-instant"],
  xai: ["grok-4.1-fast", "grok-2-latest"],
  openai: ["gpt-4.1-mini", "gpt-4o-mini", "gpt-4o"],
};

// Check if running inside Tauri
export const isTauri = (): boolean => {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
};

export const tauriApi = {
  async getSettings(): Promise<AppSettings> {
    if (!isTauri()) {
      const saved = localStorage.getItem("zyloflow_settings");
      return saved ? JSON.parse(saved) : DEFAULT_SETTINGS;
    }
    return invoke<AppSettings>("get_settings");
  },

  async saveSettings(settings: AppSettings): Promise<void> {
    if (!isTauri()) {
      localStorage.setItem("zyloflow_settings", JSON.stringify(settings));
      return;
    }
    return invoke<void>("save_settings", { settings });
  },

  async getApiKeyConfigured(provider: string): Promise<boolean> {
    if (!isTauri()) {
      return Boolean(localStorage.getItem(`zyloflow_key_${provider}`));
    }
    return invoke<boolean>("get_api_key_configured", { provider });
  },

  async saveApiKey(provider: string, key: string): Promise<void> {
    if (!isTauri()) {
      localStorage.setItem(`zyloflow_key_${provider}`, key);
      return;
    }
    return invoke<void>("save_api_key", { provider, key });
  },

  async deleteApiKey(provider: string): Promise<void> {
    if (!isTauri()) {
      localStorage.removeItem(`zyloflow_key_${provider}`);
      return;
    }
    return invoke<void>("delete_api_key", { provider });
  },

  async testApiKey(provider: string, key?: string): Promise<ProviderValidationResult> {
    if (!isTauri()) {
      return {
        provider,
        valid: Boolean(key || localStorage.getItem(`zyloflow_key_${provider}`)),
        message: "Connected successfully (Mock dev mode)",
      };
    }
    return invoke<ProviderValidationResult>("test_api_key", { provider, key });
  },

  async getSystemStatus(): Promise<SystemStatus> {
    if (!isTauri()) {
      return {
        is_active: true,
        display_server: "wayland",
        microphone_ready: true,
        current_dictation_state: "idle",
      };
    }
    return invoke<SystemStatus>("get_system_status");
  },

  async startDictation(): Promise<void> {
    if (!isTauri()) {
      console.log("Mock start dictation");
      return;
    }
    return invoke<void>("start_dictation");
  },

  async stopDictation(): Promise<void> {
    if (!isTauri()) {
      console.log("Mock stop dictation");
      return;
    }
    return invoke<void>("stop_dictation");
  },

  async cancelDictation(): Promise<void> {
    if (!isTauri()) {
      console.log("Mock cancel dictation");
      return;
    }
    return invoke<void>("cancel_dictation");
  },

  async setAppActive(active: boolean): Promise<void> {
    if (!isTauri()) {
      return;
    }
    return invoke<void>("set_app_active", { active });
  },

  async getHistory(): Promise<TranscriptionRecord[]> {
    if (!isTauri()) {
      const saved = localStorage.getItem("zyloflow_history");
      return saved ? JSON.parse(saved) : [];
    }
    return invoke<TranscriptionRecord[]>("get_history");
  },

  async deleteHistoryItem(id: string): Promise<void> {
    if (!isTauri()) {
      const history = (await this.getHistory()).filter((item) => item.id !== id);
      localStorage.setItem("zyloflow_history", JSON.stringify(history));
      return;
    }
    return invoke<void>("delete_history_item", { id });
  },

  async clearHistory(): Promise<void> {
    if (!isTauri()) {
      localStorage.removeItem("zyloflow_history");
      return;
    }
    return invoke<void>("clear_history");
  },

  async registerHotkeys(shortcuts: {
    toggle: string;
    start_app: string;
    stop_app: string;
  }): Promise<{ success: boolean; error?: string }> {
    if (!isTauri()) {
      return { success: true };
    }
    return invoke<{ success: boolean; error?: string }>("register_hotkeys", {
      shortcuts,
    });
  },

  async checkMicrophonePermission(): Promise<{ granted: boolean; error?: string }> {
    if (!isTauri()) {
      return { granted: true };
    }
    return invoke<{ granted: boolean; error?: string }>("check_microphone_permission");
  },

  async completeOnboarding(): Promise<void> {
    if (!isTauri()) {
      const settings = await this.getSettings();
      settings.is_first_launch = false;
      await this.saveSettings(settings);
      return;
    }
    return invoke<void>("complete_onboarding");
  },

  async hideMainWindow(): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>("hide_main_window");
  },

  async hidePopupWindow(): Promise<void> {
    if (!isTauri()) return;
    return invoke<void>("hide_popup_window");
  },
};
