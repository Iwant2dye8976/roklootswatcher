import React from "react";
import { Download, RefreshCw, X, Sparkles, CheckCircle2 } from "lucide-react";
import { UpdateInfo, UpdateProgress } from "../types";

interface UpdateModalProps {
  isOpen: boolean;
  onClose: () => void;
  updateInfo: UpdateInfo | null;
  progress: UpdateProgress | null;
  isUpdating: boolean;
  onInstall: () => void;
  language: "vi" | "en";
}

export const UpdateModal: React.FC<UpdateModalProps> = ({
  isOpen,
  onClose,
  updateInfo,
  progress,
  isUpdating,
  onInstall,
  language,
}) => {
  if (!isOpen || !updateInfo) return null;

  const isEn = language === "en";
  const title = isEn ? "New Update Available" : "Có Bản Cập Nhật Mới";
  const curVer = updateInfo.current_version;
  const newVer = updateInfo.latest_version;
  const isReady = progress && progress.percent >= 100;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="relative w-full max-w-lg rounded-xl bg-binance-card border border-binance-hairline shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-binance-hairline bg-binance-elevated/50">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-binance-primary/15 border border-binance-primary/30 flex items-center justify-center text-binance-primary">
              <Sparkles className="w-4 h-4" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-binance-body tracking-tight">
                {title}
              </h3>
              <p className="text-xs text-binance-muted font-mono">
                v{curVer} → <span className="text-binance-primary font-bold">v{newVer}</span>
              </p>
            </div>
          </div>
          {!isUpdating && (
            <button
              onClick={onClose}
              className="p-1 rounded-lg text-binance-muted hover:text-binance-body hover:bg-binance-elevated transition"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>

        {/* Release Notes */}
        <div className="flex-1 overflow-y-auto p-5 space-y-3 font-sans text-xs text-binance-body">
          <div className="flex items-center justify-between text-binance-muted font-mono text-[11px] pb-1 border-b border-binance-hairline/50">
            <span>{updateInfo.release_name}</span>
            <span>{updateInfo.published_at.slice(0, 10)}</span>
          </div>

          <div className="bg-binance-elevated/40 border border-binance-hairline rounded-lg p-3 whitespace-pre-wrap leading-relaxed text-binance-body max-h-56 overflow-y-auto select-text font-mono text-[11px]">
            {updateInfo.release_notes || (isEn ? "No changelog provided." : "Không có nhật ký cập nhật.")}
          </div>

          {updateInfo.is_installer && (
            <div className="flex items-center gap-1.5 text-binance-turquoise text-[11px]">
              <CheckCircle2 className="w-3.5 h-3.5 shrink-0" />
              <span>
                {isEn
                  ? "Standard NSIS Windows Installer"
                  : "Bộ cài đặt tiêu chuẩn Windows (Setup.exe)"}
              </span>
            </div>
          )}
        </div>

        {/* Progress Bar (when updating) */}
        {isUpdating && progress && (
          <div className="px-5 py-3 border-t border-binance-hairline bg-binance-elevated/20 space-y-2">
            <div className="flex justify-between text-[11px] font-mono text-binance-muted">
              <span>
                {isReady
                  ? (isEn ? "Ready to restart and install" : "Đang chuẩn bị khởi động lại...")
                  : (isEn ? "Downloading update..." : "Đang tải bản cập nhật...")}
              </span>
              <span className="text-binance-primary font-bold">
                {progress.percent.toFixed(0)}%
              </span>
            </div>
            <div className="w-full h-2 rounded-full bg-binance-elevated overflow-hidden border border-binance-hairline">
              <div
                className="h-full bg-gradient-to-r from-binance-primary to-binance-turquoise transition-all duration-200"
                style={{ width: `${Math.min(100, Math.max(0, progress.percent))}%` }}
              />
            </div>
          </div>
        )}

        {/* Footer Actions */}
        <div className="flex items-center justify-end gap-2.5 px-5 py-3 border-t border-binance-hairline bg-binance-elevated/30">
          {!isUpdating ? (
            <>
              <button
                onClick={onClose}
                className="px-3.5 py-1.5 rounded-lg text-xs font-medium text-binance-muted hover:text-binance-body hover:bg-binance-elevated transition"
              >
                {isEn ? "Later" : "Để sau"}
              </button>
              <button
                onClick={onInstall}
                className="flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-xs font-semibold bg-binance-primary text-black hover:opacity-90 transition shadow-sm"
              >
                <Download className="w-3.5 h-3.5" />
                <span>{isEn ? "Update & Restart" : "Cập nhật & Khởi động lại"}</span>
              </button>
            </>
          ) : (
            <div className="flex items-center gap-2 text-xs text-binance-muted font-mono py-1">
              <RefreshCw className="w-3.5 h-3.5 animate-spin text-binance-primary" />
              <span>{isEn ? "Please wait..." : "Vui lòng đợi..."}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
