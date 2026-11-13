import React from "react";
import { Mic, Settings, Power, Minus, X } from "lucide-react";
import { useAppStore } from "../stores/useAppStore";
import { tauriApi } from "../services/api";

export const Header: React.FC = () => {
  const { systemStatus, setAppActive, activeTab, setActiveTab } = useAppStore();

  const handleMinimize = async () => {
    await tauriApi.hideMainWindow();
  };

  const handleClose = async () => {
    await tauriApi.hideMainWindow();
  };

  return (
    <header className="h-14 border-b border-zinc-800/80 bg-zinc-950/80 backdrop-blur-md px-4 flex items-center justify-between select-none">
      {/* Brand */}
      <div className="flex items-center gap-3">
        <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-indigo-600 via-indigo-500 to-violet-500 flex items-center justify-center shadow-lg shadow-indigo-500/20">
          <Mic className="w-4 h-4 text-white" />
        </div>
        <div className="flex items-baseline gap-2">
          <span className="font-semibold tracking-tight text-white text-base">
            ZyloFlow
          </span>
          <span className="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400">
            v0.1
          </span>
        </div>
      </div>

      {/* Middle status indicator */}
      <div className="flex items-center gap-3">
        <button
          onClick={() => setAppActive(!systemStatus.is_active)}
          className={`flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium transition-all ${
            systemStatus.is_active
              ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 hover:bg-emerald-500/20"
              : "bg-amber-500/10 text-amber-400 border border-amber-500/20 hover:bg-amber-500/20"
          }`}
          title={systemStatus.is_active ? "Click to Pause" : "Click to Resume"}
        >
          <span
            className={`w-1.5 h-1.5 rounded-full ${
              systemStatus.is_active ? "bg-emerald-400 animate-pulse" : "bg-amber-400"
            }`}
          />
          <Power className="w-3 h-3" />
          <span>{systemStatus.is_active ? "Active" : "Paused"}</span>
        </button>
      </div>

      {/* Navigation & Window controls */}
      <div className="flex items-center gap-2">
        <button
          onClick={() => setActiveTab(activeTab === "settings" ? "dashboard" : "settings")}
<<<<<<< HEAD
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
            activeTab === "settings"
              ? "bg-indigo-600 text-white shadow-sm shadow-indigo-600/30"
              : "text-zinc-300 hover:text-white hover:bg-zinc-800/70 bg-zinc-900 border border-zinc-800"
          }`}
          title={activeTab === "settings" ? "Return to Dashboard" : "Open Settings"}
        >
          <Settings className="w-3.5 h-3.5" />
          <span>{activeTab === "settings" ? "Back to Dashboard" : "Settings"}</span>
=======
          className={`p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800/70 transition-colors ${
            activeTab === "settings" ? "bg-zinc-800 text-white" : ""
          }`}
          title="Settings"
        >
          <Settings className="w-4 h-4" />
>>>>>>> a40934d8418299c48f8d2ab6a4392f2321731f03
        </button>

        <div className="h-4 w-px bg-zinc-800 mx-1" />

        <button
          onClick={handleMinimize}
          className="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-zinc-800/70 transition-colors"
          title="Minimize to Tray"
        >
          <Minus className="w-4 h-4" />
        </button>
        <button
          onClick={handleClose}
          className="p-2 rounded-lg text-zinc-400 hover:text-red-400 hover:bg-red-500/10 transition-colors"
          title="Close (Hide to Tray)"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
