import React, { useState } from "react";
import {
  Mic,
  CheckCircle2,
  Command,
  ArrowRight,
  Loader2,
  AlertCircle,
} from "lucide-react";
import { useAppStore } from "../stores/useAppStore";
import { tauriApi } from "../services/api";

export const OnboardingModal: React.FC = () => {
  const { settings, updateSettings, refreshKeyStatuses } = useAppStore();
  const [step, setStep] = useState<number>(1);

  // Step 1: Mic check
  const [micStatus, setMicStatus] = useState<"untested" | "checking" | "granted" | "error">("untested");
  const [micError, setMicError] = useState<string | null>(null);

  // Step 2: API Key
  const [selectedProvider, setSelectedProvider] = useState<string>("groq");
  const [apiKeyInput, setApiKeyInput] = useState<string>("");
  const [keySaved, setKeySaved] = useState<boolean>(false);
  const [testingKey, setTestingKey] = useState<boolean>(false);
  const [keyError, setKeyError] = useState<string | null>(null);

  // Step 3: Autostart consent
  const [enableAutostart, setEnableAutostart] = useState<boolean>(false);

  const handleTestMic = async () => {
    setMicStatus("checking");
    setMicError(null);
    try {
      // In browser or webview, request audio
      if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        stream.getTracks().forEach((t) => t.stop());
        setMicStatus("granted");
      } else {
        const res = await tauriApi.checkMicrophonePermission();
        if (res.granted) {
          setMicStatus("granted");
        } else {
          setMicStatus("error");
          setMicError(res.error || "Microphone access denied");
        }
      }
    } catch (e: unknown) {
      setMicStatus("error");
      setMicError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleSaveAndTestKey = async () => {
    if (!apiKeyInput.trim()) {
      setKeyError("Please enter an API key");
      return;
    }
    setTestingKey(true);
    setKeyError(null);
    try {
      await tauriApi.saveApiKey(selectedProvider, apiKeyInput.trim());
      const res = await tauriApi.testApiKey(selectedProvider, apiKeyInput.trim());
      if (res.valid) {
        setKeySaved(true);
        await refreshKeyStatuses();
      } else {
        setKeyError(res.message);
      }
    } catch (e: unknown) {
      setKeyError(e instanceof Error ? e.message : String(e));
    } finally {
      setTestingKey(false);
    }
  };

  const handleFinish = async () => {
    await updateSettings({
      is_first_launch: false,
      launch_at_startup: enableAutostart,
      background_mode: true,
    });
    await tauriApi.completeOnboarding();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4">
      <div className="w-full max-w-lg rounded-2xl bg-zinc-950 border border-zinc-800 shadow-2xl p-6 flex flex-col gap-6 relative overflow-hidden">
        {/* Step indicator */}
        <div className="flex items-center justify-between border-b border-zinc-800/80 pb-4">
          <div className="flex items-center gap-2">
            <div className="w-7 h-7 rounded-lg bg-indigo-600 flex items-center justify-center text-white">
              <Mic className="w-4 h-4" />
            </div>
            <span className="font-semibold text-white text-sm">Welcome to ZyloFlow</span>
          </div>

          <div className="flex items-center gap-1.5 text-xs text-zinc-500">
            <span>Step {step} of 3</span>
          </div>
        </div>

        {/* STEP 1: Microphone Permission */}
        {step === 1 && (
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-1">
              <h3 className="text-base font-semibold text-white">Microphone Access</h3>
              <p className="text-xs text-zinc-400">
                ZyloFlow captures your voice using the system microphone whenever you press the global dictation shortcut.
              </p>
            </div>

            <div className="rounded-xl bg-zinc-900/60 border border-zinc-800 p-5 flex flex-col items-center justify-center text-center gap-3">
              <div
                className={`w-14 h-14 rounded-full flex items-center justify-center ${
                  micStatus === "granted"
                    ? "bg-emerald-500/20 text-emerald-400"
                    : micStatus === "error"
                    ? "bg-red-500/20 text-red-400"
                    : "bg-indigo-600/20 text-indigo-400"
                }`}
              >
                <Mic className="w-6 h-6" />
              </div>

              <div>
                <p className="text-xs font-semibold text-zinc-200">
                  {micStatus === "granted"
                    ? "Microphone Access Granted!"
                    : micStatus === "checking"
                    ? "Checking microphone..."
                    : micStatus === "error"
                    ? "Microphone Permission Issue"
                    : "Check Microphone Permission"}
                </p>
                <p className="text-[11px] text-zinc-500 mt-0.5">
                  {micError || "Click below to test audio capture and allow permissions."}
                </p>
              </div>

              {micStatus !== "granted" ? (
                <button
                  onClick={handleTestMic}
                  disabled={micStatus === "checking"}
                  className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-medium transition-colors flex items-center gap-2"
                >
                  {micStatus === "checking" && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
                  <span>Grant / Test Access</span>
                </button>
              ) : (
                <div className="flex items-center gap-1.5 text-xs text-emerald-400">
                  <CheckCircle2 className="w-4 h-4" />
                  <span>Microphone is ready</span>
                </div>
              )}
            </div>

            <div className="flex justify-end pt-2">
              <button
                onClick={() => setStep(2)}
                className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors flex items-center gap-1.5"
              >
                <span>Continue</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        )}

        {/* STEP 2: Configure API Key */}
        {step === 2 && (
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-1">
              <h3 className="text-base font-semibold text-white">Configure AI API Key</h3>
              <p className="text-xs text-zinc-400">
                To transcribe and refine your voice, provide at least one API key. Groq provides near-instant Whisper transcription!
              </p>
            </div>

            <div className="rounded-xl bg-zinc-900/60 border border-zinc-800 p-4 flex flex-col gap-3">
              <div className="flex items-center gap-2">
                <label className="text-xs font-medium text-zinc-300">Choose Provider:</label>
                <select
                  value={selectedProvider}
                  onChange={(e) => {
                    setSelectedProvider(e.target.value);
                    setKeySaved(false);
                    setKeyError(null);
                  }}
                  className="rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-1 text-xs text-zinc-200"
                >
                  <option value="groq">Groq (Recommended - Ultra Fast)</option>
                  <option value="openai">OpenAI</option>
                  <option value="gemini">Google Gemini</option>
                  <option value="xai">xAI Grok</option>
                </select>
              </div>

              <div className="flex flex-col gap-1.5">
                <input
                  type="password"
                  value={apiKeyInput}
                  onChange={(e) => {
                    setApiKeyInput(e.target.value);
                    setKeySaved(false);
                    setKeyError(null);
                  }}
                  placeholder={`Enter your ${selectedProvider.toUpperCase()} API key...`}
                  className="w-full rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-indigo-500 font-mono"
                />

                {keyError && (
                  <div className="text-[11px] text-red-400 flex items-center gap-1">
                    <AlertCircle className="w-3.5 h-3.5" />
                    <span>{keyError}</span>
                  </div>
                )}

                {keySaved && (
                  <div className="text-[11px] text-emerald-400 flex items-center gap-1">
                    <CheckCircle2 className="w-3.5 h-3.5" />
                    <span>API Key saved & verified in secure keyring!</span>
                  </div>
                )}
              </div>

              <div className="flex items-center justify-between pt-1">
                <button
                  onClick={handleSaveAndTestKey}
                  disabled={testingKey || !apiKeyInput.trim()}
                  className="px-3.5 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 text-white text-xs font-medium transition-colors flex items-center gap-1.5"
                >
                  {testingKey && <Loader2 className="w-3 h-3 animate-spin text-indigo-400" />}
                  <span>Verify & Save</span>
                </button>

                <span className="text-[10px] text-zinc-500">
                  You can add other keys anytime in Settings.
                </span>
              </div>
            </div>

            <div className="flex justify-between pt-2">
              <button
                onClick={() => setStep(1)}
                className="px-4 py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium transition-colors"
              >
                Back
              </button>

              <button
                onClick={() => setStep(3)}
                className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors flex items-center gap-1.5"
              >
                <span>Continue</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        )}

        {/* STEP 3: Hotkey & Startup Consent */}
        {step === 3 && (
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-1">
              <h3 className="text-base font-semibold text-white">Ready to Flow</h3>
              <p className="text-xs text-zinc-400">
                Dictation is ready! Here is your default global shortcut and background launch preference.
              </p>
            </div>

            {/* Hotkey Review */}
            <div className="rounded-xl bg-zinc-900/60 border border-zinc-800 p-4 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 rounded-lg bg-indigo-600/20 text-indigo-400 flex items-center justify-center">
                  <Command className="w-4 h-4" />
                </div>
                <div>
                  <p className="text-xs font-semibold text-zinc-200">Global Dictation Hotkey</p>
                  <p className="text-[11px] text-zinc-400">Press from any app to dictate</p>
                </div>
              </div>

              <span className="px-3 py-1.5 rounded-lg bg-zinc-950 border border-zinc-700 text-xs font-mono text-indigo-300">
                {settings.hotkey_toggle}
              </span>
            </div>

            {/* Explicit Startup Consent */}
            <div className="rounded-xl bg-zinc-900/60 border border-zinc-800 p-4 flex items-start gap-3">
              <input
                type="checkbox"
                id="autostartConsent"
                checked={enableAutostart}
                onChange={(e) => setEnableAutostart(e.target.checked)}
                className="mt-1 rounded bg-zinc-800 border-zinc-700 text-indigo-600 focus:ring-0 w-4 h-4 cursor-pointer"
              />
              <label htmlFor="autostartConsent" className="text-xs cursor-pointer select-none">
                <span className="font-semibold text-zinc-200 block">
                  Launch ZyloFlow at system startup
                </span>
                <span className="text-[11px] text-zinc-400 block mt-0.5">
                  Allows ZyloFlow to start in the background upon login so your hotkeys are always available without opening the dashboard.
                </span>
              </label>
            </div>

            <div className="flex justify-between pt-2">
              <button
                onClick={() => setStep(2)}
                className="px-4 py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-xs font-medium transition-colors"
              >
                Back
              </button>

              <button
                onClick={handleFinish}
                className="px-5 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-semibold transition-colors flex items-center gap-1.5 shadow-lg shadow-emerald-600/30"
              >
                <CheckCircle2 className="w-4 h-4" />
                <span>Start Using ZyloFlow</span>
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
