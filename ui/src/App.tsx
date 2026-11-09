import React, { useState, useEffect } from "react";
import { AppStoreProvider, useAppStore } from "./store/appStore";
import { Titlebar } from "./components/Titlebar";
import { Sidebar } from "./components/Sidebar";
import { GlobalFilterBar } from "./components/GlobalFilterBar";
import { CommandPalette } from "./components/CommandPalette";
import { InspectorDrawer } from "./components/InspectorDrawer";
import { ToastContainer } from "./components/ToastContainer";
import { ConfirmDialogProvider } from "./components/ConfirmDialog";

import { DashboardView } from "./views/DashboardView";
import { BattlesView } from "./views/BattlesView";
import { InventoryView } from "./views/InventoryView";
import { SettingsView } from "./views/SettingsView";
import { OverlayView } from "./views/OverlayView";

const MainLayout: React.FC = () => {
  const { tab } = useAppStore();

  const renderActiveView = () => {
    switch (tab) {
      case 0:
        return <DashboardView />;
      case 1:
        return <BattlesView />;
      case 2:
        return <InventoryView />;
      case 3:
        return <SettingsView />;
      default:
        return <DashboardView />;
    }
  };

  return (
    <div className="flex flex-col h-screen w-screen bg-background text-foreground overflow-hidden">
      {/* Frameless Titlebar */}
      <Titlebar />

      {/* Main Body: Sidebar + Active View Area + Inspector Drawer */}
      <div className="flex flex-1 overflow-hidden relative">
        <Sidebar />

        <div className="flex flex-1 flex-col overflow-hidden bg-binance-canvas">
          <GlobalFilterBar />
          {renderActiveView()}
        </div>

        <InspectorDrawer />
      </div>

      <CommandPalette />
      <ToastContainer />
    </div>
  );
};

export const App: React.FC = () => {
  const [isOverlayRoute, setIsOverlayRoute] = useState(
    window.location.hash.startsWith("#/overlay")
  );

  useEffect(() => {
    const handleHashChange = () => {
      setIsOverlayRoute(window.location.hash.startsWith("#/overlay"));
    };
    window.addEventListener("hashchange", handleHashChange);
    return () => window.removeEventListener("hashchange", handleHashChange);
  }, []);

  if (isOverlayRoute) {
    return <OverlayView />;
  }

  return (
    <AppStoreProvider>
      <ConfirmDialogProvider>
        <MainLayout />
      </ConfirmDialogProvider>
    </AppStoreProvider>
  );
};
