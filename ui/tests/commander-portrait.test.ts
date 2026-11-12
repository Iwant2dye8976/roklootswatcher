import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const source = await readFile(
  new URL("../src/components/CommanderPortrait.tsx", import.meta.url),
  "utf8"
);

test("commander portrait stacks above its frame and wakeup badge", () => {
  const frameClass = source.match(/src=\{frameSrc\}[\s\S]*?className="([^"]+)"/)?.[1];
  const badgeClass = source.match(/isExpertised && wakeupSkillSrc[\s\S]*?className="([^"]+)"/)?.[1];
  const portraitClass = source.match(/src=\{iconSrc\}[\s\S]*?className="([^"]+)"/)?.[1];

  assert.match(frameClass ?? "", /\bz-10\b/);
  assert.match(badgeClass ?? "", /\bz-20\b/);
  assert.match(portraitClass ?? "", /\bz-30\b/);
});
