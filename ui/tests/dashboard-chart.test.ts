import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { createElement } from "react";
import type { ComponentType, SVGProps } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import type { ViteDevServer } from "vite";

let server: ViteDevServer;
let ResponsiveChartSvg: ComponentType<SVGProps<SVGSVGElement>>;
let PvpChartNode: ComponentType<
  SVGProps<SVGCircleElement> & { hasEngagement: boolean }
>;
let getPvpChartRange: (
  pointCount: number,
  zoom: number,
  centerIndex?: number
) => { start: number; end: number };

before(async () => {
  const uiRoot = fileURLToPath(new URL("../", import.meta.url));
  server = await createServer({
    root: uiRoot,
    configFile: fileURLToPath(new URL("../vite.config.ts", import.meta.url)),
    optimizeDeps: { noDiscovery: true, include: [] },
    server: { middlewareMode: true },
    appType: "custom",
  });
  ({ ResponsiveChartSvg, PvpChartNode, getPvpChartRange } =
    await server.ssrLoadModule("/src/views/DashboardView.tsx"));
});

after(async () => {
  await server.close();
});

test("keeps dashboard chart proportions when the container grows wider", () => {
  assert.equal(typeof ResponsiveChartSvg, "function");

  const markup = renderToStaticMarkup(
    createElement(ResponsiveChartSvg, {
      viewBox: "0 0 700 180",
      preserveAspectRatio: "none",
      className: "w-full h-full",
    })
  );

  assert.match(markup, /preserveAspectRatio="xMidYMid meet"/);
});

test("zooms the PvP chart around a selected point and clamps to the timeline", () => {
  assert.equal(typeof getPvpChartRange, "function");
  assert.deepEqual(getPvpChartRange(24, 2, 12), { start: 6, end: 18 });
  assert.deepEqual(getPvpChartRange(24, 2, 0), { start: 0, end: 12 });
  assert.deepEqual(getPvpChartRange(24, 2, 23), { start: 12, end: 24 });
  assert.deepEqual(getPvpChartRange(24, 4, 12), { start: 9, end: 15 });
  assert.deepEqual(getPvpChartRange(7, 2, 3), { start: 1, end: 5 });
  assert.deepEqual(getPvpChartRange(0, 4, 0), { start: 0, end: 0 });
});

test("highlights PvP engagement nodes without changing inactive node colors", () => {
  assert.equal(typeof PvpChartNode, "function");

  const active = renderToStaticMarkup(
    createElement(PvpChartNode, {
      hasEngagement: true,
      cx: 100,
      cy: 50,
      r: 3,
      fill: "#0ecb81",
    })
  );
  const inactive = renderToStaticMarkup(
    createElement(PvpChartNode, {
      hasEngagement: false,
      cx: 100,
      cy: 50,
      r: 3,
      fill: "#0ecb81",
    })
  );

  assert.match(active, /stroke="#a855f7"/);
  assert.match(inactive, /stroke="#0b0e11"/);
});
