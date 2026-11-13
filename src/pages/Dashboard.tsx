import React, { useState } from "react";
import {
  Mic,
  Square,
  Sparkles,
  Command,
  Clock,
  Copy,
  Check,
  Trash2,
  Sliders,
  AudioLines,
  AlertTriangle,
  ExternalLink,
} from "lucide-react";
import { useAppStore } from "../stores/useAppStore";

export const Dashboard: React.FC = () => {
  const {
    settings,
    systemStatus,
    history,
    toggleDictation,
    deleteHistoryItem,
    clearHistory,
    setActiveTab,
    setSettingsSection,
    configuredKeys,
  } = useAppStore();

  const [copiedId, setCopiedId] = useState<string | null>(null);

  const isRecording = systemStatus.current_dictation_state === "listening";
  const isProcessing =
    systemStatus.current_dictation_state === "transcribing" ||
    systemStatus.current_dictation_state === "refining" ||
    systemStatus.current_dictation_state === "inserting";

  const isKeyMissing =
    !configuredKeys[settings.transcription_provider] ||
    (settings.enable_refinement && !configuredKeys[settings.refinement_provider]);

  const handleCopy = async (id: string, text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 2000);
    } catch (e) {
      console.error("Clipboard copy failed", e);
    }
  };

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  };

  const formatDate = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleDateString([], { month: "short", day: "numeric" });
  };

  return (
    <div className="flex-1 flex flex-col overflow-y-auto px-6 py-5 gap-6">
      {/* Alert if API Key is missing */}
      {isKeyMissing && (
        <div className="rounded-xl bg-amber-500/10 border border-amber-500/20 p-4 flex items-center justify-between text-amber-300">
          <div className="flex items-center gap-3">
            <AlertTriangle className="w-5 h-5 text-amber-400 shrink-0" />
            <div>
              <p className="text-sm font-semibold">API Key Required</p>
              <p className="text-xs text-amber-400/80">
                Configure your API key for {settings.transcription_provider}
                {settings.enable_refinement ? ` and ${settings.refinement_provider}` : ""} to start dictating.
              </p>
            </div>
          </div>
          <button
            onClick={() => {
              setActiveTab("settings");
              setSettingsSection("ai");
            }}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-amber-500/20 hover:bg-amber-500/30 text-amber-200 text-xs font-medium transition-colors"
          >
            <span>Set Keys</span>
            <ExternalLink className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Main Hero & Quick Status Card */}
      <div className="rounded-2xl bg-zinc-900/60 border border-zinc-800/80 p-6 flex flex-col md:flex-row items-center justify-between gap-6 relative overflow-hidden backdrop-blur-sm">
        {/* Subtle background glow */}
        <div className="absolute -top-24 -left-24 w-64 h-64 bg-indigo-500/10 rounded-full blur-3xl pointer-events-none" />
        <div className="absolute -bottom-24 -right-24 w-64 h-64 bg-violet-500/10 rounded-full blur-3xl pointer-events-none" />

        {/* Left Side: Overview info */}
        <div className="flex flex-col gap-3 z-10 w-full md:w-auto">
          <div className="flex items-center gap-2">
            <span className="text-xs font-semibold uppercase tracking-wider text-zinc-400">
              Desktop Dictation
            </span>
            <span className="px-2 py-0.5 rounded-full text-[11px] font-medium bg-zinc-800 text-zinc-300 border border-zinc-700/60 flex items-center gap-1">
              <Command className="w-3 h-3 text-indigo-400" />
              {settings.hotkey_toggle}
            </span>
          </div>

          <h2 className="text-2xl font-bold tracking-tight text-white">
            {isRecording
              ? "Listening to speech..."
              : isProcessing
              ? "Processing audio..."
              : "Ready to capture speech"}
          </h2>

<<<<<<< HEAD
          {systemStatus.last_error && (
            <div className="flex items-center gap-2 p-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-300 text-xs">
              <AlertTriangle className="w-4 h-4 shrink-0 text-red-400" />
              <span>{systemStatus.last_error}</span>
            </div>
          )}

=======
>>>>>>> a40934d8418299c48f8d2ab6a4392f2321731f03
          <p className="text-xs text-zinc-400 max-w-md">
            Press your global shortcut from any input field, browser, IDE, or terminal. Text will be cleaned and typed automatically.
          </p>

          {/* Provider badges */}
          <div className="flex flex-wrap items-center gap-2 pt-1">
            <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-zinc-800/70 text-zinc-300 text-xs border border-zinc-700/50">
              <AudioLines className="w-3.5 h-3.5 text-indigo-400" />
              <span className="text-zinc-400">STT:</span>
              <span className="font-medium capitalize">{settings.transcription_provider}</span>
              <span className="text-[10px] text-zinc-500 font-mono">({settings.transcription_model})</span>
            </div>

            {settings.enable_refinement && (
              <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-zinc-800/70 text-zinc-300 text-xs border border-zinc-700/50">
                <Sparkles className="w-3.5 h-3.5 text-violet-400" />
                <span className="text-zinc-400">AI:</span>
                <span className="font-medium capitalize">{settings.refinement_provider}</span>
                <span className="text-[10px] text-zinc-500 font-mono">({settings.refinement_model})</span>
              </div>
            )}
          </div>
        </div>

        {/* Right Side: Big Interactive Dictation Button */}
        <div className="flex flex-col items-center gap-2 z-10">
          <button
            onClick={toggleDictation}
            disabled={isProcessing}
            className={`relative group w-28 h-28 rounded-full flex flex-col items-center justify-center transition-all duration-300 ${
              isRecording
                ? "bg-red-500 shadow-2xl shadow-red-500/50 scale-105"
                : isProcessing
                ? "bg-zinc-800 cursor-not-allowed opacity-80"
                : "bg-indigo-600 hover:bg-indigo-500 shadow-2xl shadow-indigo-600/30 hover:scale-105"
            }`}
          >
            {/* Pulsing ring when recording */}
            {isRecording && (
              <span className="absolute inset-0 rounded-full bg-red-400/40 animate-ping pointer-events-none" />
            )}

            {isRecording ? (
              <>
                <Square className="w-8 h-8 text-white fill-current transition-transform group-hover:scale-110" />
                <span className="text-[11px] font-semibold tracking-wide text-white mt-1">
                  STOP
                </span>
              </>
            ) : isProcessing ? (
              <>
                <Sparkles className="w-8 h-8 text-indigo-300 animate-spin" />
                <span className="text-[10px] font-semibold text-zinc-400 mt-1 uppercase">
                  WAIT
                </span>
              </>
            ) : (
              <>
                <Mic className="w-8 h-8 text-white transition-transform group-hover:scale-110" />
                <span className="text-[11px] font-semibold tracking-wide text-white mt-1">
                  DICTATE
                </span>
              </>
            )}
          </button>

          <span className="text-[11px] text-zinc-500">
            {isRecording ? "Click or press hotkey to insert" : "Click or use global hotkey"}
          </span>
        </div>
      </div>

      {/* Quick Settings & Info Bar */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
<<<<<<< HEAD
        <div
          onClick={toggleDictation}
          className="rounded-xl bg-zinc-900/40 hover:bg-zinc-900/70 border border-zinc-800/80 p-3.5 flex items-center justify-between cursor-pointer transition-colors group"
          title="Click to toggle microphone dictation"
        >
          <div className="flex items-center gap-3">
            <div className={`w-9 h-9 rounded-lg flex items-center justify-center transition-colors ${
              isRecording ? "bg-red-500/20 text-red-400" : "bg-zinc-800/80 text-emerald-400 group-hover:bg-indigo-600/20 group-hover:text-indigo-400"
            }`}>
              <Mic className="w-4 h-4" />
            </div>
            <div>
              <p className="text-xs font-semibold text-zinc-200">Microphone</p>
              <p className="text-[11px] text-zinc-400">
                {isRecording ? "Listening... (Click to stop)" : "System Mic (Click to dictate)"}
              </p>
            </div>
          </div>
          <span className={`text-[11px] font-medium px-2 py-0.5 rounded ${
            isRecording ? "bg-red-500/20 text-red-400 animate-pulse" : "bg-zinc-800 text-zinc-400 group-hover:text-indigo-300"
          }`}>
            {isRecording ? "Active" : "Toggle"}
          </span>
=======
        <div className="rounded-xl bg-zinc-900/40 border border-zinc-800/80 p-3.5 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-zinc-800/80 flex items-center justify-center text-zinc-400">
              <Mic className="w-4 h-4 text-emerald-400" />
            </div>
            <div>
              <p className="text-xs font-semibold text-zinc-200">Microphone</p>
              <p className="text-[11px] text-zinc-400">System Default Mic (Ready)</p>
            </div>
          </div>
>>>>>>> a40934d8418299c48f8d2ab6a4392f2321731f03
        </div>

        <div className="rounded-xl bg-zinc-900/40 border border-zinc-800/80 p-3.5 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-zinc-800/80 flex items-center justify-center text-zinc-400">
              <Command className="w-4 h-4 text-indigo-400" />
            </div>
            <div>
              <p className="text-xs font-semibold text-zinc-200">Display Server</p>
              <p className="text-[11px] text-zinc-400 uppercase font-mono">
                {systemStatus.display_server} (Linux)
              </p>
            </div>
          </div>
        </div>

        <div className="rounded-xl bg-zinc-900/40 border border-zinc-800/80 p-3.5 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-zinc-800/80 flex items-center justify-center text-zinc-400">
              <Sliders className="w-4 h-4 text-violet-400" />
            </div>
            <div>
              <p className="text-xs font-semibold text-zinc-200">Refinement Mode</p>
              <p className="text-[11px] text-zinc-400 capitalize">
                {settings.enable_refinement ? `${settings.refinement_mode} correction` : "Disabled (Raw text)"}
              </p>
            </div>
          </div>
          <button
            onClick={() => {
              setActiveTab("settings");
              setSettingsSection("ai");
            }}
            className="text-xs text-indigo-400 hover:text-indigo-300 transition-colors"
          >
            Edit
          </button>
        </div>
      </div>

      {/* Transcription History Section */}
      <div className="flex-1 flex flex-col gap-3 min-h-[220px]">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Clock className="w-4 h-4 text-zinc-400" />
            <h3 className="text-sm font-semibold text-zinc-200">Recent Transcriptions</h3>
            <span className="text-xs px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-400 font-mono">
              {history.length}
            </span>
          </div>

          {history.length > 0 && (
            <button
              onClick={clearHistory}
              className="text-xs text-zinc-500 hover:text-red-400 transition-colors flex items-center gap-1"
            >
              <Trash2 className="w-3 h-3" />
              <span>Clear History</span>
            </button>
          )}
        </div>

        {history.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center rounded-xl bg-zinc-900/20 border border-dashed border-zinc-800/80 p-8 text-center">
            <Mic className="w-8 h-8 text-zinc-600 mb-2" />
            <p className="text-sm font-medium text-zinc-400">No transcriptions yet</p>
            <p className="text-xs text-zinc-600 mt-1">
              Press <kbd className="px-1.5 py-0.5 bg-zinc-800 rounded text-zinc-400 font-mono">{settings.hotkey_toggle}</kbd> anywhere on your system to start dictating.
            </p>
          </div>
        ) : (
          <div className="flex flex-col gap-2">
            {history.slice(0, 15).map((item) => (
              <div
                key={item.id}
                className="group rounded-xl bg-zinc-900/50 border border-zinc-800/80 p-3.5 hover:border-zinc-700/80 transition-all flex flex-col gap-2"
              >
                <div className="flex items-center justify-between text-[11px] text-zinc-500">
                  <div className="flex items-center gap-2">
                    <span>{formatDate(item.timestamp)}</span>
                    <span>•</span>
                    <span>{formatTime(item.timestamp)}</span>
                    <span>•</span>
                    <span>{item.duration_seconds.toFixed(1)}s</span>
                    <span>•</span>
                    <span className="text-zinc-400 capitalize">
                      {item.transcription_provider}
                      {item.refinement_provider ? ` + ${item.refinement_provider}` : ""}
                    </span>
                  </div>

                  <div className="flex items-center gap-1 opacity-80 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => handleCopy(item.id, item.refined_text || item.raw_text)}
                      className="p-1 rounded text-zinc-400 hover:text-white hover:bg-zinc-800 transition-colors"
                      title="Copy to clipboard"
                    >
                      {copiedId === item.id ? (
                        <Check className="w-3.5 h-3.5 text-emerald-400" />
                      ) : (
                        <Copy className="w-3.5 h-3.5" />
                      )}
                    </button>
                    <button
                      onClick={() => deleteHistoryItem(item.id)}
                      className="p-1 rounded text-zinc-400 hover:text-red-400 hover:bg-zinc-800 transition-colors"
                      title="Delete"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>

                {/* Refined Text */}
                <p className="text-xs text-zinc-200 leading-relaxed font-sans select-text">
                  {item.refined_text || item.raw_text}
                </p>

                {/* Raw diff if refinement was enabled and differed */}
                {item.refined_text && item.raw_text && item.refined_text !== item.raw_text && (
                  <details className="text-[11px] text-zinc-500 cursor-pointer">
                    <summary className="hover:text-zinc-400 transition-colors">
                      View raw transcript
                    </summary>
                    <p className="mt-1 p-2 rounded bg-zinc-950/50 border border-zinc-800 text-zinc-400 italic select-text">
                      "{item.raw_text}"
                    </p>
                  </details>
                )}

                <div className="flex items-center gap-3 text-[10px] text-zinc-500">
                  <span>{item.word_count} words</span>
                  <span>{item.char_count} chars</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
