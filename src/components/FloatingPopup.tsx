import React, { useEffect } from "react";
import {
  Mic,
  Loader2,
  Sparkles,
  CheckCircle2,
  AlertCircle,
  X,
  Keyboard,
  Square,
} from "lucide-react";
import { useAppStore } from "../stores/useAppStore";
import { tauriApi } from "../services/api";

export const FloatingPopup: React.FC = () => {
  const { systemStatus, toggleDictation, cancelDictation } = useAppStore();
  const state = systemStatus.current_dictation_state;

  // Listen for Escape key to cancel
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        cancelDictation();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [cancelDictation]);

  const handleStop = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await toggleDictation();
  };

  const handleDismiss = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await cancelDictation();
    await tauriApi.hidePopupWindow();
  };

  return (
    <div className="w-full h-full flex items-center justify-center p-2 bg-transparent select-none">
      <div className="relative group flex items-center gap-3 px-4 py-2.5 rounded-full bg-zinc-950/90 backdrop-blur-xl border border-zinc-800/90 shadow-2xl shadow-black/80 text-zinc-100 max-w-sm transition-all duration-200">
        {/* State Icon & Indicator */}
        {state === "listening" && (
          <div className="relative flex items-center justify-center">
            <span className="absolute w-8 h-8 rounded-full bg-red-500/30 animate-ping" />
            <div className="w-7 h-7 rounded-full bg-red-500 flex items-center justify-center z-10 shadow-lg shadow-red-500/50">
              <Mic className="w-3.5 h-3.5 text-white" />
            </div>
          </div>
        )}

        {state === "transcribing" && (
          <div className="w-7 h-7 rounded-full bg-indigo-500/20 text-indigo-400 flex items-center justify-center">
            <Loader2 className="w-4 h-4 animate-spin text-indigo-400" />
          </div>
        )}

        {state === "refining" && (
          <div className="w-7 h-7 rounded-full bg-violet-500/20 text-violet-400 flex items-center justify-center">
            <Sparkles className="w-4 h-4 animate-pulse text-violet-400" />
          </div>
        )}

        {state === "inserting" && (
          <div className="w-7 h-7 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center">
            <Keyboard className="w-4 h-4 text-blue-400 animate-bounce" />
          </div>
        )}

        {state === "completed" && (
          <div className="w-7 h-7 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center">
            <CheckCircle2 className="w-4 h-4 text-emerald-400" />
          </div>
        )}

        {state === "error" && (
          <div className="w-7 h-7 rounded-full bg-red-500/20 text-red-400 flex items-center justify-center">
            <AlertCircle className="w-4 h-4 text-red-400" />
          </div>
        )}

        {state === "idle" && (
          <div className="w-7 h-7 rounded-full bg-zinc-800 text-zinc-400 flex items-center justify-center">
            <Mic className="w-3.5 h-3.5" />
          </div>
        )}

        {/* Content text */}
        <div className="flex flex-col min-w-[120px] max-w-[180px] overflow-hidden">
          {state === "listening" && (
            <div className="flex flex-col">
              <div className="flex items-center gap-1.5">
                <span className="text-xs font-semibold text-white tracking-wide">
                  Listening...
                </span>
                {/* Audio wave bars */}
                <div className="flex items-center gap-0.5 h-3 ml-1">
                  <span className="w-0.5 h-2 bg-red-400 rounded-full animate-wave-bar" style={{ animationDelay: "0ms" }} />
                  <span className="w-0.5 h-3 bg-red-400 rounded-full animate-wave-bar" style={{ animationDelay: "150ms" }} />
                  <span className="w-0.5 h-1.5 bg-red-400 rounded-full animate-wave-bar" style={{ animationDelay: "300ms" }} />
                  <span className="w-0.5 h-3.5 bg-red-400 rounded-full animate-wave-bar" style={{ animationDelay: "450ms" }} />
                </div>
              </div>
              <span className="text-[10px] text-zinc-400 truncate">
                Speak now or press shortcut to stop
              </span>
            </div>
          )}

          {state === "transcribing" && (
            <div className="flex flex-col">
              <span className="text-xs font-semibold text-indigo-300">
                Transcribing...
              </span>
              <span className="text-[10px] text-zinc-400 truncate">
                Converting speech to text
              </span>
            </div>
          )}

          {state === "refining" && (
            <div className="flex flex-col">
              <span className="text-xs font-semibold text-violet-300">
                Refining...
              </span>
              <span className="text-[10px] text-zinc-400 truncate">
                Cleaning up text with AI
              </span>
            </div>
          )}

          {state === "inserting" && (
            <div className="flex flex-col">
              <span className="text-xs font-semibold text-blue-300">
                Inserting...
              </span>
              <span className="text-[10px] text-zinc-400 truncate">
                Pasting into active field
              </span>
            </div>
          )}

          {state === "completed" && (
            <div className="flex flex-col">
              <span className="text-xs font-semibold text-emerald-400">
                Done!
              </span>
              <span className="text-[10px] text-zinc-400 truncate">
                {systemStatus.last_inserted_text || "Text inserted successfully"}
              </span>
            </div>
          )}

          {state === "error" && (
            <div className="flex flex-col">
              <span className="text-xs font-semibold text-red-400">
                Error
              </span>
              <span className="text-[10px] text-zinc-400 truncate" title={systemStatus.last_error}>
                {systemStatus.last_error || "Dictation failed"}
              </span>
            </div>
          )}

          {state === "idle" && (
            <div className="flex flex-col">
              <span className="text-xs font-semibold text-zinc-300">
                ZyloFlow
              </span>
              <span className="text-[10px] text-zinc-500">
                Ready to dictate
              </span>
            </div>
          )}
        </div>

        {/* Action controls */}
        <div className="flex items-center gap-1 pl-1 border-l border-zinc-800">
          {state === "listening" && (
            <button
              onClick={handleStop}
              className="p-1 rounded-full bg-red-500/20 hover:bg-red-500/30 text-red-300 transition-colors"
              title="Stop Recording"
            >
              <Square className="w-3.5 h-3.5 fill-current" />
            </button>
          )}

          <button
            onClick={handleDismiss}
            className="p-1 rounded-full text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/60 transition-colors"
            title="Cancel / Dismiss"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
};
