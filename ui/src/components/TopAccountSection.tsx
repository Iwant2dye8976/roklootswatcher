import React, { useState, useEffect, useRef } from "react";
import {
  LogIn,
  LogOut,
  ArrowUpCircle,
  ArrowDownCircle,
  ChevronDown,
  Settings,
  AlertCircle,
  Loader2,
} from "lucide-react";
import { useAppStore } from "../store/appStore";
import { useI18n } from "../i18n";
import * as api from "../api/tauri";

export const TopAccountSection: React.FC = () => {
  const { snapshot, language, setTab, addToast, refreshSnapshot } = useAppStore();
  const { t } = useI18n(language);

  const [isOpen, setIsOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  const cloud = snapshot?.cloud_status;
  const isLoggedIn = Boolean(cloud?.logged_in);
  const isSyncing = Boolean(cloud?.is_syncing || cloud?.is_restoring);
  const isLoggingIn = Boolean(cloud?.login_in_progress);

  useEffect(() => {
    if (!isOpen) return;
    const onClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") setIsOpen(false);
    };
    document.addEventListener("mousedown", onClick);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onClick);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [isOpen]);

  const handleSignIn = async () => {
    try {
      await api.startGoogleLogin();
      addToast({
        type: "info",
        text:
          language === "vi"
            ? "Đã mở trình duyệt đăng nhập Google..."
            : "Opened Google sign-in in browser...",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleSignOut = async () => {
    try {
      await api.cloudLogout();
      await refreshSnapshot();
      setIsOpen(false);
      addToast({
        type: "info",
        text: language === "vi" ? "Đã đăng xuất" : "Signed out",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleBackup = async () => {
    try {
      await api.triggerCloudPush();
      addToast({
        type: "info",
        text: language === "vi" ? "Đang sao lưu dữ liệu..." : "Backing up records...",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const handleRestore = async () => {
    try {
      await api.triggerCloudPull();
      addToast({
        type: "info",
        text: language === "vi" ? "Đang tải dữ liệu..." : "Restoring records...",
      });
    } catch (e) {
      addToast({ type: "error", text: String(e) });
    }
  };

  const email = cloud?.email || "";
  const initial = email ? email.charAt(0).toUpperCase() : "U";

  if (!isLoggedIn) {
    return (
      <div data-no-drag>
        <button
          onClick={handleSignIn}
          disabled={isLoggingIn}
          className="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-semibold transition bg-binance-yellowSolid hover:bg-binance-yellowHover text-binance-ink shadow-xs disabled:opacity-60"
          title={t("cloudLogin")}
        >
          {isLoggingIn ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin text-binance-ink" />
          ) : (
            <LogIn className="w-3.5 h-3.5 text-binance-ink" />
          )}
          <span>{isLoggingIn ? t("cloudLoggingIn") : t("topAccountSignIn")}</span>
        </button>
      </div>
    );
  }

  return (
    <div ref={menuRef} className="relative inline-block" data-no-drag>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={`flex items-center gap-2 px-2 py-1 rounded-md text-[11px] font-medium transition border ${
          isOpen
            ? "bg-binance-elevated border-binance-yellow/50 text-binance-body"
            : "bg-binance-card hover:bg-binance-elevated border-binance-hairline text-binance-body"
        }`}
      >
        <div className="relative">
          <div className="w-5 h-5 rounded-full bg-binance-yellow/20 text-binance-yellow font-bold text-[10px] flex items-center justify-center border border-binance-yellow/40">
            {initial}
          </div>
          <span
            className={`absolute -bottom-0.5 -right-0.5 w-2 h-2 rounded-full border border-binance-canvas ${
              isSyncing
                ? "bg-binance-yellow animate-pulse"
                : cloud?.sync_error
                ? "bg-binance-red"
                : "bg-binance-green"
            }`}
          />
        </div>

        <span className="font-mono text-[11px] max-w-[120px] truncate">
          {email.split("@")[0] || email}
        </span>

        <ChevronDown
          className={`w-3 h-3 text-binance-muted transition-transform duration-150 ${
            isOpen ? "rotate-180" : ""
          }`}
        />
      </button>

      {isOpen && (
        <div className="absolute right-0 top-full mt-1.5 w-64 rounded-lg bg-binance-card border border-binance-hairline shadow-2xl p-3 text-xs z-50 space-y-2.5">
          <div className="border-b border-binance-hairline pb-2">
            <div className="font-medium text-binance-body text-[12px] truncate" title={email}>
              {email}
            </div>
            {cloud?.last_sync_time && (
              <div className="text-[10px] text-binance-muted font-mono mt-0.5">
                {t("cloudLastSync")} {cloud.last_sync_time}
              </div>
            )}
          </div>

          {cloud?.sync_error && (
            <div className="flex items-start gap-1.5 p-2 rounded bg-binance-red/10 border border-binance-red/30 text-[11px] text-binance-red font-mono">
              <AlertCircle className="w-3.5 h-3.5 shrink-0 mt-0.5" />
              <span className="break-all">{cloud.sync_error}</span>
            </div>
          )}

          <div className="grid grid-cols-2 gap-1.5">
            <button
              onClick={handleBackup}
              disabled={isSyncing}
              className="flex items-center justify-center gap-1.5 py-1.5 px-2 rounded bg-binance-elevated hover:bg-binance-hairline text-binance-body border border-binance-hairline text-[11px] transition disabled:opacity-50"
            >
              <ArrowUpCircle className="w-3.5 h-3.5 text-binance-green" />
              <span>{t("topAccountSyncNow")}</span>
            </button>
            <button
              onClick={handleRestore}
              disabled={isSyncing}
              className="flex items-center justify-center gap-1.5 py-1.5 px-2 rounded bg-binance-elevated hover:bg-binance-hairline text-binance-body border border-binance-hairline text-[11px] transition disabled:opacity-50"
            >
              <ArrowDownCircle className="w-3.5 h-3.5 text-binance-info" />
              <span>{t("topAccountRestore")}</span>
            </button>
          </div>

          <div className="pt-2 border-t border-binance-hairline flex items-center justify-between">
            <button
              onClick={() => {
                setTab(3);
                setIsOpen(false);
              }}
              className="flex items-center gap-1.5 text-[11px] text-binance-muted hover:text-binance-body transition py-1 px-1.5 rounded hover:bg-binance-elevated"
            >
              <Settings className="w-3.5 h-3.5" />
              <span>{t("topAccountSettings")}</span>
            </button>

            <button
              onClick={handleSignOut}
              className="flex items-center gap-1.5 text-[11px] text-binance-red hover:text-red-400 transition py-1 px-1.5 rounded hover:bg-binance-red/10"
            >
              <LogOut className="w-3.5 h-3.5" />
              <span>{t("cloudLogout")}</span>
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
