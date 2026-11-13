import React, { useEffect, useState } from "react";
import { Header } from "./components/Header";
import { Dashboard } from "./pages/Dashboard";
import { SettingsPage } from "./pages/Settings";
import { FloatingPopup } from "./components/FloatingPopup";
import { OnboardingModal } from "./components/OnboardingModal";
import { useAppStore } from "./stores/useAppStore";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { isTauri } from "./services/api";

export const App: React.FC = () => {
  const {
    settings,
    activeTab,
    loadInitialData,
    setupEventListeners,
    isLoading,
  } = useAppStore();

  const [isPopupWindow, setIsPopupWindow] = useState<boolean>(false);

  useEffect(() => {
    // Detect if this instance is the floating popup
    const searchParams = new URLSearchParams(window.location.search);
    const isPopupParam = searchParams.get("popup") === "true";
    const isPopupHash = window.location.hash.includes("popup");

    let isPopupLabel = false;
    if (isTauri()) {
      try {
        const appWindow = getCurrentWebviewWindow();
        if (appWindow.label === "popup") {
          isPopupLabel = true;
        }
      } catch (e) {
        console.debug("Could not determine window label", e);
      }
    }

    setIsPopupWindow(isPopupParam || isPopupHash || isPopupLabel);

    loadInitialData();
    const cleanupPromise = setupEventListeners();

    return () => {
      cleanupPromise.then((cleanup) => cleanup && cleanup());
    };
  }, [loadInitialData, setupEventListeners]);

  // If this is the floating popup window, render ONLY the floating popup
  if (isPopupWindow) {
    return <FloatingPopup />;
  }

  if (isLoading) {
    return (
      <div className="w-screen h-screen flex items-center justify-center bg-zinc-950 text-zinc-400">
        <div className="flex flex-col items-center gap-3">
          <div className="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-xs font-mono">Loading ZyloFlow...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="w-screen h-screen flex flex-col bg-zinc-950 text-zinc-100 overflow-hidden font-sans select-none">
      <Header />
      <main className="flex-1 flex flex-col overflow-hidden">
        {activeTab === "dashboard" ? <Dashboard /> : <SettingsPage />}
      </main>

      {/* First-time Onboarding Modal */}
      {settings.is_first_launch && <OnboardingModal />}
    </div>
  );
};
