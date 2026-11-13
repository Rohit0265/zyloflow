import React, { useState } from "react";
import {
  Sparkles,
  Keyboard,
  Sliders,
  Eye,
  EyeOff,
  CheckCircle2,
  AlertCircle,
  Loader2,
  RotateCcw,
  KeyRound,
  Layers,
  Info,
} from "lucide-react";
import { useAppStore } from "../stores/useAppStore";
import {
  tauriApi,
  PROVIDER_MODELS,
  REFINEMENT_MODELS,
} from "../services/api";
import type {
  TranscriptionProviderId,
  RefinementProviderId,
  RefinementMode,
  ClipboardFallbackMode,
} from "../types";

export const SettingsPage: React.FC = () => {
  const {
    settings,
    updateSettings,
    settingsSection,
    setSettingsSection,
    configuredKeys,
    refreshKeyStatuses,
  } = useAppStore();

  // Local state for keys input
  const [keysInput, setKeysInput] = useState<Record<string, string>>({
    groq: "",
    openai: "",
    gemini: "",
    xai: "",
  });
  const [showKeys, setShowKeys] = useState<Record<string, boolean>>({});
  const [testingStatus, setTestingStatus] = useState<
    Record<string, { loading: boolean; valid?: boolean; message?: string }>
  >({});
  const [recordingShortcutFor, setRecordingShortcutFor] = useState<string | null>(null);
  const [hotkeyError, setHotkeyError] = useState<string | null>(null);

  const toggleShowKey = (provider: string) => {
    setShowKeys((prev) => ({ ...prev, [provider]: !prev[provider] }));
  };

  const handleKeyChange = (provider: string, val: string) => {
    setKeysInput((prev) => ({ ...prev, [provider]: val }));
  };

  const handleSaveKey = async (provider: string) => {
    const val = keysInput[provider];
    if (val !== undefined && val.trim() !== "") {
      await tauriApi.saveApiKey(provider, val.trim());
      await refreshKeyStatuses();
      setKeysInput((prev) => ({ ...prev, [provider]: "" }));
    }
  };

  const handleDeleteKey = async (provider: string) => {
    await tauriApi.deleteApiKey(provider);
    await refreshKeyStatuses();
  };

  const handleTestKey = async (provider: string) => {
    setTestingStatus((prev) => ({
      ...prev,
      [provider]: { loading: true },
    }));

    try {
      const keyToTest = keysInput[provider]?.trim() || undefined;
      const res = await tauriApi.testApiKey(provider, keyToTest);
      setTestingStatus((prev) => ({
        ...prev,
        [provider]: {
          loading: false,
          valid: res.valid,
          message: res.message,
        },
      }));
      if (res.valid) {
        await refreshKeyStatuses();
      }
    } catch (err: unknown) {
      setTestingStatus((prev) => ({
        ...prev,
        [provider]: {
          loading: false,
          valid: false,
          message: err instanceof Error ? err.message : String(err),
        },
      }));
    }
  };

  // Keyboard capture for shortcuts
  const startRecordingShortcut = (type: "toggle" | "start" | "stop") => {
    setRecordingShortcutFor(type);
    setHotkeyError(null);
  };

  const handleKeyDownCapture = (e: React.KeyboardEvent) => {
    if (!recordingShortcutFor) return;

    e.preventDefault();
    e.stopPropagation();

    // Ignore standalone modifier presses
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) {
      return;
    }

    if (e.key === "Escape") {
      setRecordingShortcutFor(null);
      return;
    }

    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Control");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Super");

    let keyName = e.key;
    if (keyName === " ") keyName = "Space";
    else if (keyName.length === 1) keyName = keyName.toUpperCase();

    parts.push(keyName);
    const shortcutStr = parts.join("+");

    // Duplicate check
    const currentShortcuts = {
      toggle: settings.hotkey_toggle,
      start: settings.hotkey_start_app,
      stop: settings.hotkey_stop_app,
    };

    const isDuplicate = Object.entries(currentShortcuts).some(
      ([key, val]) => key !== recordingShortcutFor && val.toLowerCase() === shortcutStr.toLowerCase()
    );

    if (isDuplicate) {
      setHotkeyError(`Shortcut "${shortcutStr}" is already assigned in ZyloFlow.`);
      setRecordingShortcutFor(null);
      return;
    }

    if (recordingShortcutFor === "toggle") {
      updateSettings({ hotkey_toggle: shortcutStr });
    } else if (recordingShortcutFor === "start") {
      updateSettings({ hotkey_start_app: shortcutStr });
    } else if (recordingShortcutFor === "stop") {
      updateSettings({ hotkey_stop_app: shortcutStr });
    }

    setRecordingShortcutFor(null);

    // Persist and register with native backend
    tauriApi.registerHotkeys({
      toggle: recordingShortcutFor === "toggle" ? shortcutStr : settings.hotkey_toggle,
      start_app: recordingShortcutFor === "start" ? shortcutStr : settings.hotkey_start_app,
      stop_app: recordingShortcutFor === "stop" ? shortcutStr : settings.hotkey_stop_app,
    }).then((res) => {
      if (!res.success && res.error) {
        setHotkeyError(`Native registration notice: ${res.error}`);
      }
    });
  };

  const handleResetShortcuts = async () => {
    await updateSettings({
      hotkey_toggle: "Control+Alt+Space",
      hotkey_start_app: "Control+Alt+S",
      hotkey_stop_app: "Control+Alt+X",
    });
    await tauriApi.registerHotkeys({
      toggle: "Control+Alt+Space",
      start_app: "Control+Alt+S",
      stop_app: "Control+Alt+X",
    });
    setHotkeyError(null);
  };

  return (
    <div
      className="flex-1 flex flex-col md:flex-row overflow-hidden"
      onKeyDown={recordingShortcutFor ? handleKeyDownCapture : undefined}
      tabIndex={recordingShortcutFor ? 0 : undefined}
    >
      {/* Sidebar Tabs */}
      <div className="w-full md:w-56 border-r border-zinc-800/80 bg-zinc-950/40 p-4 flex md:flex-col gap-1 shrink-0">
        <button
          onClick={() => setSettingsSection("ai")}
          className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors ${
            settingsSection === "ai"
              ? "bg-indigo-600/20 text-indigo-300 border border-indigo-500/30"
              : "text-zinc-400 hover:text-white hover:bg-zinc-800/50"
          }`}
        >
          <Sparkles className="w-4 h-4 text-indigo-400" />
          <span>AI Providers</span>
        </button>

        <button
          onClick={() => setSettingsSection("hotkeys")}
          className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors ${
            settingsSection === "hotkeys"
              ? "bg-indigo-600/20 text-indigo-300 border border-indigo-500/30"
              : "text-zinc-400 hover:text-white hover:bg-zinc-800/50"
          }`}
        >
          <Keyboard className="w-4 h-4 text-indigo-400" />
          <span>Hotkeys</span>
        </button>

        <button
          onClick={() => setSettingsSection("general")}
          className={`flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors ${
            settingsSection === "general"
              ? "bg-indigo-600/20 text-indigo-300 border border-indigo-500/30"
              : "text-zinc-400 hover:text-white hover:bg-zinc-800/50"
          }`}
        >
          <Sliders className="w-4 h-4 text-indigo-400" />
          <span>General</span>
        </button>
      </div>

      {/* Main Settings Content Area */}
      <div className="flex-1 overflow-y-auto p-6 flex flex-col gap-6">
        {/* ===================== AI PROVIDERS ===================== */}
        {settingsSection === "ai" && (
          <div className="flex flex-col gap-6 max-w-2xl">
            <div>
              <h3 className="text-base font-semibold text-white">AI Provider Configuration</h3>
              <p className="text-xs text-zinc-400 mt-0.5">
                Configure your Speech-to-Text engine and AI text refinement models. Keys are stored in the OS-native secure keyring.
              </p>
            </div>

            {/* STT Configuration Card */}
            <div className="rounded-xl bg-zinc-900/50 border border-zinc-800/80 p-5 flex flex-col gap-4">
              <div className="flex items-center justify-between border-b border-zinc-800 pb-3">
                <div className="flex items-center gap-2">
                  <Layers className="w-4 h-4 text-indigo-400" />
                  <span className="text-sm font-semibold text-zinc-200">Speech-to-Text (STT)</span>
                </div>
                <span className="text-[11px] text-zinc-400 font-mono">Whisper Engine</span>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                {/* Provider Select */}
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-medium text-zinc-300">Provider</label>
                  <select
                    value={settings.transcription_provider}
                    onChange={(e) => {
                      const p = e.target.value as TranscriptionProviderId;
                      updateSettings({
                        transcription_provider: p,
                        transcription_model: PROVIDER_MODELS[p][0],
                      });
                    }}
                    className="rounded-lg bg-zinc-950 border border-zinc-700/80 px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-indigo-500"
                  >
                    <option value="groq">Groq (Ultra-Fast Whisper)</option>
                    <option value="openai">OpenAI Whisper</option>
                  </select>
                </div>

                {/* Model Select */}
                <div className="flex flex-col gap-1.5">
                  <label className="text-xs font-medium text-zinc-300">Model</label>
                  <select
                    value={settings.transcription_model}
                    onChange={(e) => updateSettings({ transcription_model: e.target.value })}
                    className="rounded-lg bg-zinc-950 border border-zinc-700/80 px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-indigo-500 font-mono"
                  >
                    {PROVIDER_MODELS[settings.transcription_provider].map((m) => (
                      <option key={m} value={m}>
                        {m}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            </div>

            {/* AI Refinement Configuration Card */}
            <div className="rounded-xl bg-zinc-900/50 border border-zinc-800/80 p-5 flex flex-col gap-4">
              <div className="flex items-center justify-between border-b border-zinc-800 pb-3">
                <div className="flex items-center gap-2">
                  <Sparkles className="w-4 h-4 text-violet-400" />
                  <span className="text-sm font-semibold text-zinc-200">Text Refinement (LLM)</span>
                </div>
                <div className="flex items-center gap-2">
                  <label className="text-xs text-zinc-400 cursor-pointer flex items-center gap-1.5">
                    <input
                      type="checkbox"
                      checked={settings.enable_refinement}
                      onChange={(e) => updateSettings({ enable_refinement: e.target.checked })}
                      className="rounded bg-zinc-800 border-zinc-700 text-indigo-600 focus:ring-0"
                    />
                    <span>Enable AI Refinement</span>
                  </label>
                </div>
              </div>

              {settings.enable_refinement && (
                <>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    {/* Provider Select */}
                    <div className="flex flex-col gap-1.5">
                      <label className="text-xs font-medium text-zinc-300">LLM Provider</label>
                      <select
                        value={settings.refinement_provider}
                        onChange={(e) => {
                          const p = e.target.value as RefinementProviderId;
                          updateSettings({
                            refinement_provider: p,
                            refinement_model: REFINEMENT_MODELS[p][0],
                          });
                        }}
                        className="rounded-lg bg-zinc-950 border border-zinc-700/80 px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-indigo-500"
                      >
                        <option value="gemini">Google Gemini (Developer API)</option>
                        <option value="groq">Groq LLM</option>
                        <option value="xai">xAI Grok</option>
                        <option value="openai">OpenAI GPT</option>
                      </select>
                    </div>

                    {/* Model Select */}
                    <div className="flex flex-col gap-1.5">
                      <label className="text-xs font-medium text-zinc-300">Model</label>
                      <select
                        value={settings.refinement_model}
                        onChange={(e) => updateSettings({ refinement_model: e.target.value })}
                        className="rounded-lg bg-zinc-950 border border-zinc-700/80 px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-indigo-500 font-mono"
                      >
                        {REFINEMENT_MODELS[settings.refinement_provider].map((m) => (
                          <option key={m} value={m}>
                            {m}
                          </option>
                        ))}
                      </select>
                    </div>
                  </div>

                  {/* Refinement Mode */}
                  <div className="flex flex-col gap-1.5">
                    <label className="text-xs font-medium text-zinc-300">Correction Mode</label>
                    <div className="grid grid-cols-3 gap-2">
                      {(["minimal", "standard", "professional"] as RefinementMode[]).map((mode) => (
                        <button
                          key={mode}
                          onClick={() => updateSettings({ refinement_mode: mode })}
                          className={`px-3 py-2 rounded-lg text-xs font-medium capitalize border transition-all ${
                            settings.refinement_mode === mode
                              ? "bg-violet-600/20 text-violet-300 border-violet-500/50"
                              : "bg-zinc-950 text-zinc-400 border-zinc-800 hover:text-white"
                          }`}
                        >
                          {mode}
                        </button>
                      ))}
                    </div>
                    <span className="text-[11px] text-zinc-500">
                      {settings.refinement_mode === "minimal" && "Fixes only severe typos and punctuation."}
                      {settings.refinement_mode === "standard" && "Removes filler words and cleans grammar naturally."}
                      {settings.refinement_mode === "professional" && "Polishes phrasing for work emails, documents, and code."}
                    </span>
                  </div>
                </>
              )}
            </div>

            {/* API Keys Management */}
            <div className="rounded-xl bg-zinc-900/50 border border-zinc-800/80 p-5 flex flex-col gap-4">
              <div className="flex items-center justify-between border-b border-zinc-800 pb-3">
                <div className="flex items-center gap-2">
                  <KeyRound className="w-4 h-4 text-emerald-400" />
                  <span className="text-sm font-semibold text-zinc-200">API Credentials</span>
                </div>
                <div className="flex items-center gap-1.5 text-[11px] text-zinc-400">
                  <Info className="w-3.5 h-3.5 text-zinc-500" />
                  <span>Single key shared if same provider is used for both STT and LLM</span>
                </div>
              </div>

              {/* Provider Key Inputs */}
              {(["groq", "openai", "gemini", "xai"] as const).map((prov) => {
                const isConfigured = Boolean(configuredKeys[prov]);
                const testState = testingStatus[prov];

                return (
                  <div
                    key={prov}
                    className="flex flex-col gap-2 p-3 rounded-lg bg-zinc-950/60 border border-zinc-800/60"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="text-xs font-semibold uppercase text-zinc-200">
                          {prov}
                        </span>
                        {isConfigured ? (
                          <span className="flex items-center gap-1 text-[10px] text-emerald-400 font-medium px-2 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/20">
                            <CheckCircle2 className="w-3 h-3" />
                            Saved in Keyring
                          </span>
                        ) : (
                          <span className="text-[10px] text-zinc-500 font-medium px-2 py-0.5 rounded bg-zinc-800">
                            Not Configured
                          </span>
                        )}
                      </div>

                      {isConfigured && (
                        <button
                          onClick={() => handleDeleteKey(prov)}
                          className="text-[11px] text-zinc-500 hover:text-red-400 transition-colors"
                        >
                          Remove Key
                        </button>
                      )}
                    </div>

                    <div className="flex items-center gap-2">
                      <div className="relative flex-1">
                        <input
                          type={showKeys[prov] ? "text" : "password"}
                          value={keysInput[prov] || ""}
                          onChange={(e) => handleKeyChange(prov, e.target.value)}
                          placeholder={isConfigured ? "•••••••••••••••••••••••• (Leave blank to keep)" : `Enter ${prov.toUpperCase()} API Key`}
                          className="w-full rounded-lg bg-zinc-900 border border-zinc-700/80 px-3 py-1.5 pr-8 text-xs text-zinc-200 focus:outline-none focus:border-indigo-500 font-mono"
                        />
                        <button
                          type="button"
                          onClick={() => toggleShowKey(prov)}
                          className="absolute right-2.5 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300"
                        >
                          {showKeys[prov] ? (
                            <EyeOff className="w-3.5 h-3.5" />
                          ) : (
                            <Eye className="w-3.5 h-3.5" />
                          )}
                        </button>
                      </div>

                      <button
                        onClick={() => handleSaveKey(prov)}
                        disabled={!keysInput[prov]?.trim()}
                        className="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed text-white text-xs font-medium transition-colors"
                      >
                        Save
                      </button>

                      <button
                        onClick={() => handleTestKey(prov)}
                        disabled={testState?.loading}
                        className="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-zinc-200 text-xs font-medium transition-colors flex items-center gap-1.5"
                      >
                        {testState?.loading ? (
                          <Loader2 className="w-3 h-3 animate-spin text-indigo-400" />
                        ) : null}
                        <span>Test</span>
                      </button>
                    </div>

                    {/* Test result message */}
                    {testState && (
                      <div
                        className={`text-[11px] flex items-center gap-1.5 mt-1 ${
                          testState.valid ? "text-emerald-400" : "text-red-400"
                        }`}
                      >
                        {testState.valid ? (
                          <CheckCircle2 className="w-3.5 h-3.5" />
                        ) : (
                          <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                        )}
                        <span>{testState.message}</span>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* ===================== HOTKEYS ===================== */}
        {settingsSection === "hotkeys" && (
          <div className="flex flex-col gap-6 max-w-2xl">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-base font-semibold text-white">Global Hotkeys</h3>
                <p className="text-xs text-zinc-400 mt-0.5">
                  Configure desktop-wide keyboard shortcuts. Shortcuts trigger native OS events without opening the dashboard window.
                </p>
              </div>

              <button
                onClick={handleResetShortcuts}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors"
              >
                <RotateCcw className="w-3.5 h-3.5" />
                <span>Reset Defaults</span>
              </button>
            </div>

            {hotkeyError && (
              <div className="rounded-xl bg-amber-500/10 border border-amber-500/20 p-3.5 flex items-center gap-2 text-xs text-amber-300">
                <AlertCircle className="w-4 h-4 text-amber-400 shrink-0" />
                <span>{hotkeyError}</span>
              </div>
            )}

            <div className="flex flex-col gap-3">
              {/* Hotkey 1: Toggle Dictation */}
              <div className="rounded-xl bg-zinc-900/50 border border-zinc-800/80 p-4 flex items-center justify-between">
                <div>
                  <p className="text-sm font-semibold text-zinc-200">Toggle Dictation</p>
                  <p className="text-xs text-zinc-400">
                    Starts microphone recording when inactive; stops and inserts text when active.
                  </p>
                </div>

                <div className="flex items-center gap-2">
                  <span className="px-3 py-1.5 rounded-lg bg-zinc-950 border border-zinc-700/80 text-xs font-mono text-indigo-300">
                    {recordingShortcutFor === "toggle" ? (
                      <span className="text-amber-400 animate-pulse font-sans font-medium">
                        Press shortcut keys...
                      </span>
                    ) : (
                      settings.hotkey_toggle
                    )}
                  </span>
                  <button
                    onClick={() => startRecordingShortcut("toggle")}
                    className="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-xs text-zinc-200 font-medium transition-colors"
                  >
                    {recordingShortcutFor === "toggle" ? "Cancel (Esc)" : "Record"}
                  </button>
                </div>
              </div>

              {/* Hotkey 2: Start Application */}
              <div className="rounded-xl bg-zinc-900/50 border border-zinc-800/80 p-4 flex items-center justify-between">
                <div>
                  <p className="text-sm font-semibold text-zinc-200">Start / Resume Application</p>
                  <p className="text-xs text-zinc-400">
                    Activates ZyloFlow if paused without bringing up the dashboard window.
                  </p>
                </div>

                <div className="flex items-center gap-2">
                  <span className="px-3 py-1.5 rounded-lg bg-zinc-950 border border-zinc-700/80 text-xs font-mono text-zinc-300">
                    {recordingShortcutFor === "start" ? (
                      <span className="text-amber-400 animate-pulse font-sans font-medium">
                        Press shortcut keys...
                      </span>
                    ) : (
                      settings.hotkey_start_app
                    )}
                  </span>
                  <button
                    onClick={() => startRecordingShortcut("start")}
                    className="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-xs text-zinc-200 font-medium transition-colors"
                  >
                    {recordingShortcutFor === "start" ? "Cancel (Esc)" : "Record"}
                  </button>
                </div>
              </div>

              {/* Hotkey 3: Stop Application */}
              <div className="rounded-xl bg-zinc-900/50 border border-zinc-800/80 p-4 flex items-center justify-between">
                <div>
                  <p className="text-sm font-semibold text-zinc-200">Stop / Pause Application</p>
                  <p className="text-xs text-zinc-400">
                    Stops any ongoing dictation and pauses ZyloFlow activity.
                  </p>
                </div>

                <div className="flex items-center gap-2">
                  <span className="px-3 py-1.5 rounded-lg bg-zinc-950 border border-zinc-700/80 text-xs font-mono text-zinc-300">
                    {recordingShortcutFor === "stop" ? (
                      <span className="text-amber-400 animate-pulse font-sans font-medium">
                        Press shortcut keys...
                      </span>
                    ) : (
                      settings.hotkey_stop_app
                    )}
                  </span>
                  <button
                    onClick={() => startRecordingShortcut("stop")}
                    className="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-xs text-zinc-200 font-medium transition-colors"
                  >
                    {recordingShortcutFor === "stop" ? "Cancel (Esc)" : "Record"}
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* ===================== GENERAL ===================== */}
        {settingsSection === "general" && (
          <div className="flex flex-col gap-6 max-w-2xl">
            <div>
              <h3 className="text-base font-semibold text-white">General Application Settings</h3>
              <p className="text-xs text-zinc-400 mt-0.5">
                Customize background behavior, system startup, and text insertion preferences.
              </p>
            </div>

            <div className="rounded-xl bg-zinc-900/50 border border-zinc-800/80 p-5 flex flex-col gap-5">
              {/* Launch at startup */}
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-semibold text-zinc-200">Launch at Startup</p>
                  <p className="text-[11px] text-zinc-400">
                    Automatically launch ZyloFlow in the background when logging into Linux.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.launch_at_startup}
                  onChange={(e) => updateSettings({ launch_at_startup: e.target.checked })}
                  className="rounded bg-zinc-800 border-zinc-700 text-indigo-600 focus:ring-0 w-4 h-4 cursor-pointer"
                />
              </div>

              <div className="h-px bg-zinc-800/80" />

              {/* Start Minimized to Tray */}
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-semibold text-zinc-200">Start Minimized to System Tray</p>
                  <p className="text-[11px] text-zinc-400">
                    Keep dashboard window hidden on launch; accessible via the tray icon.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.start_minimized}
                  onChange={(e) => updateSettings({ start_minimized: e.target.checked })}
                  className="rounded bg-zinc-800 border-zinc-700 text-indigo-600 focus:ring-0 w-4 h-4 cursor-pointer"
                />
              </div>

              <div className="h-px bg-zinc-800/80" />

              {/* Enable Background Mode */}
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-semibold text-zinc-200">Background Mode</p>
                  <p className="text-[11px] text-zinc-400">
                    Closing the window keeps ZyloFlow running silently in the system tray.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.background_mode}
                  onChange={(e) => updateSettings({ background_mode: e.target.checked })}
                  className="rounded bg-zinc-800 border-zinc-700 text-indigo-600 focus:ring-0 w-4 h-4 cursor-pointer"
                />
              </div>

              <div className="h-px bg-zinc-800/80" />

              {/* Transcription History Toggle */}
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-semibold text-zinc-200">Save Transcription History</p>
                  <p className="text-[11px] text-zinc-400">
                    Save recent dictations locally on your machine for easy re-copying.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.enable_history}
                  onChange={(e) => updateSettings({ enable_history: e.target.checked })}
                  className="rounded bg-zinc-800 border-zinc-700 text-indigo-600 focus:ring-0 w-4 h-4 cursor-pointer"
                />
              </div>

              <div className="h-px bg-zinc-800/80" />

              {/* Audio Language Selection */}
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-semibold text-zinc-200">Audio Language</p>
                  <p className="text-[11px] text-zinc-400">
                    Spoken language for Whisper transcription.
                  </p>
                </div>
                <select
                  value={settings.audio_language}
                  onChange={(e) => updateSettings({ audio_language: e.target.value })}
                  className="rounded-lg bg-zinc-950 border border-zinc-700/80 px-3 py-1.5 text-xs text-zinc-200 focus:outline-none focus:border-indigo-500"
                >
                  <option value="auto">Auto-Detect</option>
                  <option value="en">English</option>
                  <option value="es">Spanish</option>
                  <option value="fr">French</option>
                  <option value="de">German</option>
                  <option value="it">Italian</option>
                  <option value="pt">Portuguese</option>
                  <option value="ja">Japanese</option>
                  <option value="zh">Chinese</option>
                  <option value="hi">Hindi</option>
                </select>
              </div>

              <div className="h-px bg-zinc-800/80" />

              {/* Text Insertion Fallback Mode */}
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-semibold text-zinc-200">Text Insertion Method</p>
                  <p className="text-[11px] text-zinc-400">
                    How text is inserted into external applications (e.g., Wayland / X11).
                  </p>
                </div>
                <select
                  value={settings.clipboard_fallback_mode}
                  onChange={(e) =>
                    updateSettings({
                      clipboard_fallback_mode: e.target.value as ClipboardFallbackMode,
                    })
                  }
                  className="rounded-lg bg-zinc-950 border border-zinc-700/80 px-3 py-1.5 text-xs text-zinc-200 focus:outline-none focus:border-indigo-500"
                >
                  <option value="auto">Auto (Native with Clipboard Fallback)</option>
                  <option value="clipboard_only">Clipboard Only</option>
                  <option value="native_preferred">Native OS Injection</option>
                </select>
              </div>

              <div className="h-px bg-zinc-800/80" />

              {/* System Notifications */}
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-xs font-semibold text-zinc-200">System Notifications</p>
                  <p className="text-[11px] text-zinc-400">
                    Display desktop notifications for dictation errors or completion.
                  </p>
                </div>
                <input
                  type="checkbox"
                  checked={settings.enable_notifications}
                  onChange={(e) => updateSettings({ enable_notifications: e.target.checked })}
                  className="rounded bg-zinc-800 border-zinc-700 text-indigo-600 focus:ring-0 w-4 h-4 cursor-pointer"
                />
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
