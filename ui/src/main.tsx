import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import "./index.css";

interface ErrorBoundaryProps {
  children: React.ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
}

class RootErrorBoundary extends React.Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error("RootErrorBoundary caught:", error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex flex-col items-center justify-center h-screen w-screen bg-[#0b0e11] text-[#eaecef] p-6 font-mono select-none">
          <div className="max-w-md w-full bg-[#1e2329] border border-[#f6465d]/50 rounded-lg p-5 shadow-2xl space-y-3">
            <div className="text-sm font-bold text-[#f6465d] uppercase tracking-wider">
              Application Render Error
            </div>
            <div className="text-xs text-[#707a8a] break-words bg-[#0b0e11] p-3 rounded border border-[#2b3139]">
              {this.state.error?.message || "An unexpected error occurred."}
            </div>
            <button
              onClick={() => window.location.reload()}
              className="w-full py-2 bg-[#fcd535] hover:bg-[#f0b90b] text-[#181a20] font-bold text-xs rounded transition"
            >
              Reload Application
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <RootErrorBoundary>
      <App />
    </RootErrorBoundary>
  </React.StrictMode>
);
