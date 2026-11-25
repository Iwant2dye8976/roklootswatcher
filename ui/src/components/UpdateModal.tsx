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
  const pct = progress ? Math.min(100, Math.max(0, progress.percent)) : 0;
  const isReady = progress?.status === "ready_to_install" || pct >= 100;
  const isVerifying = progress?.status === "verifying_checksum";
  const formatMb = (bytes: number) => `${(bytes / (1024 * 1024)).toFixed(1)} MB`;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 select-none">
      <div className="relative w-full max-w-lg rounded-xl bg-binance-card border border-binance-hairline shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-binance-hairline bg-binance-elevated/50">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-binance-yellow/15 border border-binance-yellow/30 flex items-center justify-center text-binance-yellow">
              <Sparkles className="w-4 h-4" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-binance-body tracking-tight">
                {title}
              </h3>
              <p className="text-xs text-binance-muted font-mono">
                v{curVer} → <span className="text-binance-yellow font-bold">v{newVer}</span>
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
            <span className="font-medium text-binance-body">{updateInfo.release_name}</span>
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
        {isUpdating && (
          <div className="px-5 py-3 border-t border-binance-hairline bg-binance-elevated/30 space-y-2">
            <div className="flex justify-between items-center text-[11px] font-mono text-binance-muted gap-2">
              <span>
                {isReady
                  ? (isEn ? "Ready to restart and install" : "Đang chuẩn bị khởi động lại...")
                  : isVerifying
                  ? (isEn ? "Verifying SHA256 checksum..." : "Đang kiểm tra mã SHA256...")
                  : progress
                  ? (isEn ? "Downloading update..." : "Đang tải bản cập nhật...")
                  : (isEn ? "Connecting to release server..." : "Đang kết nối máy chủ...")}
              </span>
              <div className="flex items-center gap-2 shrink-0">
                {progress && progress.downloaded_bytes > 0 && (
                  <span className="text-binance-muted">
                    {progress.total_bytes > 0
                      ? `${formatMb(progress.downloaded_bytes)} / ${formatMb(progress.total_bytes)}`
                      : formatMb(progress.downloaded_bytes)}
                  </span>
                )}
                <span className="text-binance-yellow font-bold">
                  {pct.toFixed(0)}%
                </span>
              </div>
            </div>
            <div className="w-full h-2.5 rounded-full bg-binance-elevated overflow-hidden border border-binance-hairline">
              {progress && progress.total_bytes === 0 && !isReady ? (
                <div className="h-full w-1/3 rounded-full bg-gradient-to-r from-binance-yellowSolid to-binance-turquoise animate-pulse" />
              ) : (
                <div
                  className="h-full rounded-full bg-gradient-to-r from-binance-yellowSolid to-binance-turquoise transition-all duration-150"
                  style={{ width: `${pct}%` }}
                />
              )}
            </div>
          </div>
        )}

        {/* Footer Actions */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-binance-hairline bg-binance-elevated/40">
          {!isUpdating ? (
            <>
              <button
                type="button"
                onClick={onClose}
                className="px-5 py-2.5 rounded-lg text-xs font-semibold text-binance-body bg-binance-card hover:bg-binance-elevated border border-binance-hairline hover:border-binance-hairline/80 active:scale-[0.98] transition shadow-sm cursor-pointer"
              >
                {isEn ? "Later" : "Để sau"}
              </button>
              <button
                type="button"
                onClick={onInstall}
                className="flex items-center gap-2 px-5 py-2.5 rounded-lg text-xs font-bold bg-binance-yellowSolid text-binance-ink hover:bg-binance-yellowHover active:scale-[0.98] transition shadow-lg shadow-binance-yellowSolid/20 hover:shadow-binance-yellowSolid/30 cursor-pointer"
              >
                <Download className="w-4 h-4 stroke-[2.5]" />
                <span>{isEn ? "Update & Restart" : "Cập nhật & Khởi động lại"}</span>
              </button>
            </>
          ) : (
            <div className="flex items-center gap-2.5 text-xs text-binance-body font-mono py-1.5 font-medium">
              <RefreshCw className="w-4 h-4 animate-spin text-binance-yellowSolid" />
              <span>{isEn ? "Downloading update, please wait..." : "Đang tải bản cập nhật, vui lòng đợi..."}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
