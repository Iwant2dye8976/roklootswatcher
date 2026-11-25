import React from "react";
import { CheckCircle2, AlertCircle, Info, AlertTriangle, X } from "lucide-react";
import { useAppStore } from "../store/appStore";

export const ToastContainer: React.FC = () => {
  const { toasts, removeToast } = useAppStore();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-12 left-1/2 -translate-x-1/2 z-50 flex flex-col items-center gap-2 max-w-md w-full px-4 pointer-events-none">
      {toasts.map((toast) => {
        const Icon =
          toast.type === "success"
            ? CheckCircle2
            : toast.type === "error"
            ? AlertCircle
            : toast.type === "warning"
            ? AlertTriangle
            : Info;

        const borderBg =
          toast.type === "success"
            ? "border-emerald-300 bg-emerald-50 text-emerald-800 dark:border-emerald-500/40 dark:bg-[#0c1a17] dark:text-emerald-300"
            : toast.type === "error"
            ? "border-red-300 bg-red-50 text-red-800 dark:border-red-500/40 dark:bg-[#1f1012] dark:text-red-300"
            : toast.type === "warning"
            ? "border-amber-300 bg-amber-50 text-amber-900 dark:border-amber-500/40 dark:bg-[#1c160c] dark:text-amber-300"
            : "border-teal-300 bg-teal-50 text-teal-800 dark:border-teal-500/40 dark:bg-[#0d1720] dark:text-teal-300";

        return (
          <div
            key={toast.id}
            className={`pointer-events-auto flex items-start gap-2.5 p-3 rounded-lg border shadow-lg text-xs backdrop-blur-md animate-in slide-in-from-top-2 duration-150 w-full ${borderBg}`}
          >
            <Icon className="w-4 h-4 shrink-0 mt-0.5" />
            <div className="flex-1 font-medium leading-relaxed">{toast.text}</div>
            <button
              onClick={() => removeToast(toast.id)}
              className="p-0.5 rounded hover:bg-black/5 dark:hover:bg-white/10 text-binance-muted hover:text-binance-body transition"
              aria-label="Close"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        );
      })}
    </div>
  );
};

