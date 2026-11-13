import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { createElement } from "react";
import type { ComponentType } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import type { ViteDevServer } from "vite";

let server: ViteDevServer;
let ResourceIcon: ComponentType<{
  resource?: string | number | null;
  size?: "xs" | "sm" | "md" | "lg" | "xl" | number;
  className?: string;
}>;

before(async () => {
  const uiRoot = fileURLToPath(new URL("../", import.meta.url));
  server = await createServer({
    root: uiRoot,
    configFile: fileURLToPath(new URL("../vite.config.ts", import.meta.url)),
    server: { middlewareMode: true },
    appType: "custom",
  });
  ({ ResourceIcon } = await server.ssrLoadModule(
    "/src/components/ResourceIcon.tsx"
  ));
});

after(async () => {
  await server.close();
});

test("shows a fallback glyph when a catalog sprite has no frontend mapping", () => {
  const markup = renderToStaticMarkup(
    createElement(ResourceIcon, { resource: "img_KVK_Coin2big.png" })
  );

  assert.match(markup, /<svg/);
});
