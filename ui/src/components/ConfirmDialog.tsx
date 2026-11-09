import React, { createContext, useContext, useState, useCallback } from "react";
import { AlertTriangle, Trash2, X } from "lucide-react";

export interface ConfirmDialogOptions {
  title?: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  isDestructive?: boolean;
}

interface ConfirmDialogContextType {
  confirm: (options: ConfirmDialogOptions) => Promise<boolean>;
}

const ConfirmDialogContext = createContext<ConfirmDialogContextType | null>(null);

export const ConfirmDialogProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [dialog, setDialog] = useState<{
    options: ConfirmDialogOptions;
    resolve: (value: boolean) => void;
  } | null>(null);

  const confirm = useCallback((options: ConfirmDialogOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      setDialog({ options, resolve });
    });
  }, []);

  const handleClose = (result: boolean) => {
    if (dialog) {
      dialog.resolve(result);
      setDialog(null);
    }
  };

  return (
    <ConfirmDialogContext.Provider value={{ confirm }}>
      {children}
      {dialog && (
        <div
          className="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/65 backdrop-blur-sm animate-in fade-in duration-150 select-none"
          onClick={() => handleClose(false)}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            className="w-full max-w-md bg-binance-card border border-binance-hairline rounded-lg shadow-2xl p-5 text-binance-body flex flex-col gap-4 animate-in zoom-in-95 duration-150 font-sans"
          >
            {/* Header */}
            <div className="flex items-start justify-between gap-3">
              <div className="flex items-center gap-3">
                <div
                  className={`w-9 h-9 rounded-md flex items-center justify-center shrink-0 border ${
                    dialog.options.isDestructive
                      ? "bg-binance-red/15 border-binance-red/40 text-binance-red"
                      : "bg-binance-yellow/15 border-binance-yellow/40 text-binance-yellow"
                  }`}
                >
                  {dialog.options.isDestructive ? (
                    <Trash2 className="w-5 h-5" />
                  ) : (
                    <AlertTriangle className="w-5 h-5" />
                  )}
                </div>
                <div>
                  <h3 className="text-sm font-bold font-mono text-binance-body tracking-wide">
                    {dialog.options.title || "Confirm Action"}
                  </h3>
                  <div className="text-[10px] text-binance-muted uppercase font-mono tracking-wider">
                    {dialog.options.isDestructive ? "Irreversible Operation" : "Notice"}
                  </div>
                </div>
              </div>
              <button
                onClick={() => handleClose(false)}
                className="p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated transition"
                aria-label="Close"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Content Message */}
            <div className="text-xs text-binance-body/90 leading-relaxed pl-12 pr-1">
              {dialog.options.message}
            </div>

            {/* Actions */}
            <div className="flex items-center justify-end gap-2 pt-2 border-t border-binance-hairline/60">
              <button
                type="button"
                onClick={() => handleClose(false)}
                className="px-3.5 py-1.5 rounded text-xs font-mono font-medium bg-binance-canvas hover:bg-binance-elevated border border-binance-hairline text-binance-body transition"
              >
                {dialog.options.cancelLabel || "Cancel"}
              </button>
              <button
                type="button"
                onClick={() => handleClose(true)}
                autoFocus
                className={`px-3.5 py-1.5 rounded text-xs font-mono font-semibold transition ${
                  dialog.options.isDestructive
                    ? "bg-binance-red hover:bg-binance-red/90 text-white shadow-sm shadow-binance-red/30"
                    : "bg-binance-yellowSolid hover:bg-binance-yellowSolid/90 text-binance-ink font-bold shadow-sm"
                }`}
              >
                {dialog.options.confirmLabel || "Confirm"}
              </button>
            </div>
          </div>
        </div>
      )}
    </ConfirmDialogContext.Provider>
  );
};

export const useConfirm = () => {
  const context = useContext(ConfirmDialogContext);
  if (!context) {
    throw new Error("useConfirm must be used within a ConfirmDialogProvider");
  }
  return context;
};
