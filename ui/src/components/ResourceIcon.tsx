import React, { useState } from "react";
import { Package } from "lucide-react";
import {
  getResourceIconLayers,
  ResourceItemInput,
} from "../utils/resources";

export interface ResourceIconProps {
  resource?: ResourceItemInput | string | number | null;
  subType?: number;
  size?: "xs" | "sm" | "md" | "lg" | "xl" | number;
  className?: string;
  alt?: string;
  fallback?: React.ReactNode;
}

export const ResourceIcon: React.FC<ResourceIconProps> = ({
  resource,
  subType,
  size = "sm",
  className = "",
  alt = "",
  fallback,
}) => {
  const [failedSources, setFailedSources] = useState<string[]>([]);

  let sizeClasses = "w-4 h-4";
  let inlineStyle: React.CSSProperties | undefined = undefined;

  if (typeof size === "number") {
    inlineStyle = { width: `${size}px`, height: `${size}px` };
    sizeClasses = "";
  } else {
    switch (size) {
      case "xs":
        sizeClasses = "w-3.5 h-3.5";
        break;
      case "sm":
        sizeClasses = "w-4 h-4";
        break;
      case "md":
        sizeClasses = "w-5 h-5";
        break;
      case "lg":
        sizeClasses = "w-6 h-6";
        break;
      case "xl":
        sizeClasses = "w-8 h-8";
        break;
    }
  }

  const iconSources = (getResourceIconLayers(resource, subType) || []).filter(
    (source) => !failedSources.includes(source)
  );

  if (iconSources.length === 0) {
    if (fallback !== undefined) return <>{fallback}</>;

    return (
      <span
        style={inlineStyle}
        className={`inline-flex shrink-0 select-none pointer-events-none ${sizeClasses} ${className}`}
        role={alt ? "img" : undefined}
        aria-label={alt || undefined}
      >
        <Package
          className="w-full h-full text-binance-muted"
          aria-hidden="true"
        />
      </span>
    );
  }

  const handleImageError = (source: string) => {
    setFailedSources((current) =>
      current.includes(source) ? current : [...current, source]
    );
  };

  return (
    <span
      style={inlineStyle}
      className={`relative inline-block shrink-0 select-none pointer-events-none ${sizeClasses} ${className}`}
      role={alt ? "img" : undefined}
      aria-label={alt || undefined}
    >
      {iconSources.map((source, index) => (
        <img
          key={`${source}-${index}`}
          src={source}
          alt=""
          className="absolute inset-0 w-full h-full object-contain"
          loading="lazy"
          decoding="async"
          onError={() => handleImageError(source)}
        />
      ))}
    </span>
  );
};
