import React, { useState } from "react";
import {
  getCommanderMeta,
  getCommanderBackgroundUrl,
  getCommanderFrameUrl,
  getCommanderIconUrl,
} from "../utils/commanders";

export type CommanderCardSize = "xs" | "sm" | "md" | "lg" | "xl";

export interface CommanderCardPairProps {
  // Primary (main) commander
  primaryId?: number;
  primaryName?: string;
  primaryIsExpertised?: boolean;

  // Secondary commander
  secondaryId?: number;
  secondaryName?: string;
  secondaryIsExpertised?: boolean;

  // Display options
  size?: CommanderCardSize;
  className?: string;
  showWakeupBadge?: boolean;
  showEmblem?: boolean;
  title?: string;
}

// Preset sizing definitions (width and height for the combined container)
interface SizeDimensions {
  container: string;
  primaryWidth: number;
  primaryHeight: number;
  secondaryWidth: number;
  secondaryHeight: number;
  secondaryOffsetLeft: number;
  secondaryOffsetTop: number;
}

const SIZE_CONFIGS: Record<CommanderCardSize, SizeDimensions> = {
  xs: {
    container: "w-[50px] h-[36px]",
    primaryWidth: 32,
    primaryHeight: 34,
    secondaryWidth: 24,
    secondaryHeight: 26,
    secondaryOffsetLeft: 22,
    secondaryOffsetTop: 4,
  },
  sm: {
    container: "w-[62px] h-[44px]",
    primaryWidth: 40,
    primaryHeight: 42,
    secondaryWidth: 30,
    secondaryHeight: 32,
    secondaryOffsetLeft: 27,
    secondaryOffsetTop: 5,
  },
  md: {
    container: "w-[74px] h-[52px]",
    primaryWidth: 48,
    primaryHeight: 50,
    secondaryWidth: 36,
    secondaryHeight: 38,
    secondaryOffsetLeft: 32,
    secondaryOffsetTop: 6,
  },
  lg: {
    container: "w-[92px] h-[64px]",
    primaryWidth: 60,
    primaryHeight: 62,
    secondaryWidth: 45,
    secondaryHeight: 47,
    secondaryOffsetLeft: 40,
    secondaryOffsetTop: 8,
  },
  xl: {
    container: "w-[114px] h-[78px]",
    primaryWidth: 74,
    primaryHeight: 76,
    secondaryWidth: 55,
    secondaryHeight: 57,
    secondaryOffsetLeft: 50,
    secondaryOffsetTop: 10,
  },
};

/**
 * Single stylized hexagonal commander card with clip path, metallic border, and inner portrait
 */
interface SingleCardProps {
  id?: number;
  name?: string;
  isExpertised?: boolean;
  width: number;
  height: number;
  isMain?: boolean;
}

const SingleCard: React.FC<SingleCardProps> = ({
  id,
  name,
  isExpertised = false,
  width,
  height,
}) => {
  const [hasError, setHasError] = useState(false);
  const meta = getCommanderMeta(id, name);
  const rarity = (meta?.rarity || "legendary").toLowerCase();

  const bgSrc = getCommanderBackgroundUrl(rarity);
  const frameSrc = getCommanderFrameUrl(rarity, isExpertised);
  const iconSrc = hasError
    ? "./commanders/img_icon_HeroProfile_BGWhite2.png"
    : getCommanderIconUrl(id, name);


  return (
    <div
      style={{
        width: `${width}px`,
        height: `${height}px`,
      }}
      className="relative shrink-0 select-none transition-transform duration-150"
    >
      {/* Hexagon Container without artificial outer gradient frame */}
      <div
        className="relative w-full h-full overflow-hidden"
      >
        {/* Layer 1: Rarity Background */}
        <img
          src={bgSrc}
          alt=""
          className="absolute inset-0 w-full h-full object-cover pointer-events-none"
          draggable={false}
        />

        {/* Layer 2: Commander Portrait */}
        <img
          src={iconSrc}
          alt={name || meta?.name || "Commander"}
          className="absolute inset-0 w-full h-full object-contain pointer-events-none scale-105"
          onError={() => setHasError(true)}
          draggable={false}
          loading="lazy"
        />

        {/* Layer 3: RoK In-game Decorative Mask Frame Overlay */}
        <img
          src={frameSrc}
          alt=""
          className="absolute inset-0 w-full h-full object-fill pointer-events-none z-10 opacity-90 mix-blend-screen"
          draggable={false}
        />
      </div>
    </div>
  );
};

export const CommanderCardPair: React.FC<CommanderCardPairProps> = ({
  primaryId,
  primaryName,
  primaryIsExpertised = false,
  secondaryId,
  secondaryName,
  secondaryIsExpertised = false,
  size = "lg",
  className = "",
  title,
}) => {
  const cfg = SIZE_CONFIGS[size];
  const primaryMeta = getCommanderMeta(primaryId, primaryName);
  const hasSecondary = !!(secondaryId || secondaryName);

  const displayPrimary = primaryName || primaryMeta?.name || (primaryId ? `Cmdr #${primaryId}` : "Unknown");
  const displaySecondary = secondaryName || (secondaryId ? `Cmdr #${secondaryId}` : undefined);
  const fullPairName = displaySecondary ? `${displayPrimary} + ${displaySecondary}` : displayPrimary;
  const tooltip = title || fullPairName;

  return (
    <div
      className={`relative inline-flex items-center shrink-0 select-none ${cfg.container} ${className}`}
      title={tooltip}
      role="img"
      aria-label={fullPairName}
    >
      {/* Secondary Commander (Layer 0: Behind, smaller, offset to the right) */}
      {hasSecondary && (
        <div
          style={{
            position: "absolute",
            left: `${cfg.secondaryOffsetLeft}px`,
            top: `${cfg.secondaryOffsetTop}px`,
            zIndex: 10,
          }}
          className="opacity-95"
        >
          <SingleCard
            id={secondaryId}
            name={secondaryName}
            isExpertised={secondaryIsExpertised}
            width={cfg.secondaryWidth}
            height={cfg.secondaryHeight}
            isMain={false}
          />
        </div>
      )}

      {/* Primary Commander (Layer 1: Foreground, larger, front-left position) */}
      <div
        style={{
          position: "absolute",
          left: 0,
          top: 0,
          zIndex: 20,
        }}
      >
        <SingleCard
          id={primaryId}
          name={primaryName}
          isExpertised={primaryIsExpertised}
          width={cfg.primaryWidth}
          height={cfg.primaryHeight}
          isMain={true}
        />
      </div>
    </div>
  );
};
