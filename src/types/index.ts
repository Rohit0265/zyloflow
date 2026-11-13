export type TranscriptionProviderId = 'groq' | 'openai';
export type RefinementProviderId = 'gemini' | 'groq' | 'xai' | 'openai';
export type RefinementMode = 'minimal' | 'standard' | 'professional';
export type ClipboardFallbackMode = 'auto' | 'clipboard_only' | 'native_preferred';

export type DictationState =
  | 'idle'
  | 'listening'
  | 'transcribing'
  | 'refining'
  | 'inserting'
  | 'completed'
  | 'error';

export interface AppSettings {
  is_first_launch: boolean;
  transcription_provider: TranscriptionProviderId;
  transcription_model: string;
  refinement_provider: RefinementProviderId;
  refinement_model: string;
  refinement_mode: RefinementMode;
  enable_refinement: boolean;
  enable_history: boolean;
  audio_language: string;
  hotkey_toggle: string;
  hotkey_start_app: string;
  hotkey_stop_app: string;
  launch_at_startup: boolean;
  start_minimized: boolean;
  background_mode: boolean;
  clipboard_fallback_mode: ClipboardFallbackMode;
  enable_notifications: boolean;
  microphone_device_id: string;
}

export interface ApiKeysConfig {
  groq_key?: string;
  openai_key?: string;
  gemini_key?: string;
  xai_key?: string;
}

export interface TranscriptionRecord {
  id: string;
  timestamp: number;
  raw_text: string;
  refined_text: string;
  duration_seconds: number;
  transcription_provider: string;
  refinement_provider: string;
  char_count: number;
  word_count: number;
}

export interface SystemStatus {
  is_active: boolean;
  display_server: 'x11' | 'wayland' | 'unknown';
  microphone_ready: boolean;
  current_dictation_state: DictationState;
  state_message?: string;
  last_inserted_text?: string;
  last_error?: string;
}

export interface ProviderValidationResult {
  provider: string;
  valid: boolean;
  message: string;
}
