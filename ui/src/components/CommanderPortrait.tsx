import React, { useState } from "react";
import {
  getCommanderMeta,
  getCommanderBackgroundUrl,
  getCommanderFrameUrl,
  getCommanderIconUrl,
  getCommanderWakeupSkillUrl,
} from "../utils/commanders";

export interface CommanderPortraitProps {
  id?: number;
  name?: string;
  isExpertised?: boolean;
  size?: "xs" | "sm" | "md" | "lg" | "xl";
  className?: string;
  title?: string;
  showWakeupBadge?: boolean;
}

export const CommanderPortrait: React.FC<CommanderPortraitProps> = ({
  id,
  name,
  isExpertised = false,
  size = "sm",
  className = "",
  title,
  showWakeupBadge = true,
}) => {
  const [hasError, setHasError] = useState(false);
  const meta = getCommanderMeta(id, name);
  const rarity = meta?.rarity || "legendary";

  const bgSrc = getCommanderBackgroundUrl(rarity);
  const frameSrc = getCommanderFrameUrl(rarity, isExpertised);
  const iconSrc = hasError
    ? "./commanders/img_icon_HeroProfile_BGWhite2.png"
    : getCommanderIconUrl(id, name);
  const wakeupSkillSrc = getCommanderWakeupSkillUrl(meta?.wakeupSkillSprite);

  const sizeClass =
    size === "xs"
      ? "w-8 h-8"
      : size === "sm"
      ? "w-10 h-10"
      : size === "md"
      ? "w-12 h-12"
      : size === "lg"
      ? "w-16 h-16"
      : "w-20 h-20";

  const displayName = name || meta?.name || (id ? `Cmdr #${id}` : "Commander");
  const fullTitle =
    title ||
    `${displayName}${isExpertised ? " (Expertised)" : ""}${
      isExpertised && meta?.wakeupSkillName ? ` • ${meta.wakeupSkillName}` : ""
    }`;

  return (
    <div
      className={`relative inline-flex items-center justify-center shrink-0 select-none ${sizeClass} ${className}`}
      title={fullTitle}
    >
      {/* Layer 1: Background (by commander rarity: legendary / epic / elite / advanced) */}
      <img
        src={bgSrc}
        alt=""
        className="absolute inset-0 w-full h-full object-contain pointer-events-none"
        draggable={false}
      />

      {/* Layer 2: Hero Portrait */}
      <img
        src={iconSrc}
        alt={displayName}
        className="absolute inset-0 w-full h-full object-contain pointer-events-none transition-transform duration-150"
        onError={() => setHasError(true)}
        draggable={false}
        loading="lazy"
      />

      {/* Layer 3: Hero Frame (Mask_Orange / Mask_pink if expertised, Mask otherwise) */}
      <img
        src={frameSrc}
        alt=""
        className="absolute inset-0 w-full h-full object-contain pointer-events-none z-10"
        draggable={false}
      />

      {/* Layer 4: Wakeup Skill Badge (shown when commander is expertised and has a wakeup skill) */}
      {isExpertised && wakeupSkillSrc && showWakeupBadge && (
        <div
          className="absolute -bottom-0.5 -right-0.5 z-20 w-[36%] h-[36%] rounded-full overflow-hidden border border-amber-400 bg-black/90 shadow-md ring-1 ring-black/40 pointer-events-none"
          title={meta?.wakeupSkillName ? `Expertise: ${meta.wakeupSkillName}` : "Expertised"}
        >
          <img
            src={wakeupSkillSrc}
            alt=""
            className="w-full h-full object-cover"
            draggable={false}
          />
        </div>
      )}
    </div>
  );
};
