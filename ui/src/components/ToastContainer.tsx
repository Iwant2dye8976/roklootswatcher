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
            ? "border-emerald-500/40 bg-[#0c1a17] text-emerald-300"
            : toast.type === "error"
            ? "border-red-500/40 bg-[#1f1012] text-red-300"
            : toast.type === "warning"
            ? "border-amber-500/40 bg-[#1c160c] text-amber-300"
            : "border-teal-500/40 bg-[#0d1720] text-teal-300";

        return (
          <div
            key={toast.id}
            className={`pointer-events-auto flex items-start gap-2.5 p-3 rounded-lg border shadow-2xl text-xs backdrop-blur-md animate-in slide-in-from-top-2 duration-150 w-full ${borderBg}`}
          >
            <Icon className="w-4 h-4 shrink-0 mt-0.5" />
            <div className="flex-1 font-medium leading-relaxed">{toast.text}</div>
            <button
              onClick={() => removeToast(toast.id)}
              className="p-0.5 rounded hover:bg-white/10 text-slate-400 hover:text-white transition"
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

