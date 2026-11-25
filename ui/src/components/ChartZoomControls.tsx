import React, { useState, useCallback } from "react";
import { ChevronLeft, ChevronRight, ZoomIn, ZoomOut, RotateCcw } from "lucide-react";

export const CHART_MIN_VISIBLE_POINTS = 3;
export const CHART_MAX_ZOOM = 8;

/** Computes a clamped `[start, end)` window over `pointCount` series points for a given zoom & center. */
export function getChartRange(
  pointCount: number,
  zoom: number,
  centerIndex = (pointCount - 1) / 2
): { start: number; end: number } {
  const count = Math.max(0, Math.floor(pointCount));
  if (count === 0) return { start: 0, end: 0 };

  const safeZoom = Number.isFinite(zoom)
    ? Math.min(CHART_MAX_ZOOM, Math.max(1, zoom))
    : 1;
  const visibleCount = Math.min(
    count,
    Math.max(Math.min(CHART_MIN_VISIBLE_POINTS, count), Math.ceil(count / safeZoom))
  );
  const safeCenter = Number.isFinite(centerIndex)
    ? Math.min(count - 1, Math.max(0, centerIndex))
    : (count - 1) / 2;
  const maxStart = count - visibleCount;
  const start = Math.min(
    maxStart,
    Math.max(0, Math.round(safeCenter - visibleCount / 2))
  );

  return { start, end: start + visibleCount };
}

/** Shared zoom and pan state for dashboard time-series and level breakdown charts. */
export function useChartZoom(pointCount: number, onViewportChange?: () => void) {
  const [zoom, setZoom] = useState(1);
  const [center, setCenter] = useState<number | undefined>(undefined);

  const range = getChartRange(pointCount, zoom, center);
  const visibleCount = range.end - range.start;
  const effectiveZoom =
    pointCount > 0 && visibleCount > 0 ? pointCount / visibleCount : 1;
  const minVisible = Math.min(CHART_MIN_VISIBLE_POINTS, pointCount);
  const canZoomIn = visibleCount > minVisible && zoom < CHART_MAX_ZOOM;
  const canPanPrevious = range.start > 0;
  const canPanNext = range.end < pointCount;

  const zoomBy = useCallback(
    (factor: number) => {
      const nextZoom = Math.min(CHART_MAX_ZOOM, Math.max(1, zoom * factor));
      const currentCenter = (range.start + range.end) / 2;
      setZoom(nextZoom);
      setCenter(currentCenter);
      onViewportChange?.();
    },
    [zoom, range.start, range.end, onViewportChange]
  );

  const panBy = useCallback(
    (direction: -1 | 1) => {
      const shift = Math.max(1, Math.floor(visibleCount * 0.75));
      const currentCenter = (range.start + range.end) / 2;
      setCenter(currentCenter + direction * shift);
      onViewportChange?.();
    },
    [visibleCount, range.start, range.end, onViewportChange]
  );

  const reset = useCallback(() => {
    setZoom(1);
    setCenter(undefined);
    onViewportChange?.();
  }, [onViewportChange]);

  return {
    zoom,
    range,
    effectiveZoom,
    canZoomIn,
    canPanPrevious,
    canPanNext,
    zoomBy,
    panBy,
    reset,
  };
}

export interface ChartZoomLabels {
  controls: string;
  previous: string;
  zoomOut: string;
  zoomIn: string;
  reset: string;
  next: string;
}

interface ChartZoomControlsProps {
  zoomState: ReturnType<typeof useChartZoom>;
  labels: ChartZoomLabels;
  ringClassName?: string;
}

/** Toolbar rendering pan-left, zoom-out, zoom-percentage, zoom-in, reset, and pan-right buttons. */
export const ChartZoomControls: React.FC<ChartZoomControlsProps> = ({
  zoomState,
  labels,
  ringClassName = "focus-visible:ring-binance-yellow",
}) => {
  const btnCls = `p-1 rounded text-binance-muted hover:text-binance-body hover:bg-binance-elevated disabled:opacity-40 disabled:cursor-not-allowed focus-visible:outline-none focus-visible:ring-1 ${ringClassName}`;

  return (
    <div
      role="group"
      aria-label={labels.controls}
      className="flex items-center gap-0.5 rounded border border-binance-hairline bg-binance-elevated/50 px-1 py-0.5"
    >
      <button
        type="button"
        onClick={() => zoomState.panBy(-1)}
        disabled={!zoomState.canPanPrevious}
        aria-label={labels.previous}
        title={labels.previous}
        className={btnCls}
      >
        <ChevronLeft className="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onClick={() => zoomState.zoomBy(0.5)}
        disabled={zoomState.zoom <= 1}
        aria-label={labels.zoomOut}
        title={labels.zoomOut}
        className={btnCls}
      >
        <ZoomOut className="w-3.5 h-3.5" />
      </button>
      <span
        aria-live="polite"
        className="min-w-9 text-center text-[10px] font-mono text-binance-muted font-semibold"
      >
        {Math.round(zoomState.effectiveZoom * 100)}%
      </span>
      <button
        type="button"
        onClick={() => zoomState.zoomBy(2)}
        disabled={!zoomState.canZoomIn}
        aria-label={labels.zoomIn}
        title={labels.zoomIn}
        className={btnCls}
      >
        <ZoomIn className="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onClick={zoomState.reset}
        disabled={zoomState.zoom <= 1}
        aria-label={labels.reset}
        title={labels.reset}
        className={btnCls}
      >
        <RotateCcw className="w-3.5 h-3.5" />
      </button>
      <button
        type="button"
        onClick={() => zoomState.panBy(1)}
        disabled={!zoomState.canPanNext}
        aria-label={labels.next}
        title={labels.next}
        className={btnCls}
      >
        <ChevronRight className="w-3.5 h-3.5" />
      </button>
    </div>
  );
};
