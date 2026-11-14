import React, { useState, useEffect, lazy, Suspense } from "react";
import { AppStoreProvider, useAppStore } from "./store/appStore";
import { Titlebar } from "./components/Titlebar";
import { Sidebar } from "./components/Sidebar";
import { GlobalFilterBar } from "./components/GlobalFilterBar";
import { CommandPalette } from "./components/CommandPalette";
import { InspectorDrawer } from "./components/InspectorDrawer";
import { ToastContainer } from "./components/ToastContainer";
import { ConfirmDialogProvider } from "./components/ConfirmDialog";

import { DashboardView } from "./views/DashboardView";

const BattlesView = lazy(() =>
  import("./views/BattlesView").then((m) => ({ default: m.BattlesView }))
);
const InventoryView = lazy(() =>
  import("./views/InventoryView").then((m) => ({ default: m.InventoryView }))
);
const SettingsView = lazy(() =>
  import("./views/SettingsView").then((m) => ({ default: m.SettingsView }))
);
const OverlayView = lazy(() =>
  import("./views/OverlayView").then((m) => ({ default: m.OverlayView }))
);

const ViewLoadingFallback: React.FC = () => (
  <div className="flex-1 flex items-center justify-center p-8 text-neutral-400">
    <div className="w-5 h-5 border-2 border-primary border-t-transparent rounded-full animate-spin mr-3" />
    <span className="text-xs font-medium">Loading view...</span>
  </div>
);

const MainLayout: React.FC = () => {
  const { tab } = useAppStore();

  const renderActiveView = () => {
    switch (tab) {
      case 0:
        return <DashboardView />;
      case 1:
        return (
          <Suspense fallback={<ViewLoadingFallback />}>
            <BattlesView />
          </Suspense>
        );
      case 2:
        return (
          <Suspense fallback={<ViewLoadingFallback />}>
            <InventoryView />
          </Suspense>
        );
      case 3:
        return (
          <Suspense fallback={<ViewLoadingFallback />}>
            <SettingsView />
          </Suspense>
        );
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
    return (
      <Suspense fallback={null}>
        <OverlayView />
      </Suspense>
    );
  }

  return (
    <AppStoreProvider>
      <ConfirmDialogProvider>
        <MainLayout />
      </ConfirmDialogProvider>
    </AppStoreProvider>
  );
};
