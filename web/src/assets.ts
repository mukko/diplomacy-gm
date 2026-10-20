// Loads the map drawing assets (docs/en/map.md § 7) and turns them into board descriptions.
import { useEffect, useState } from "react";
import { api } from "./api";
import { inlineSvg, type BoardAsset } from "./components/Board";
import type { MapData, Point } from "./types";

export interface Assets {
  map: BoardAsset;
  graph: BoardAsset;
  data: MapData;
}

function closestLocation(el: Element): string | undefined {
  const coast = el.closest("[data-coast]");
  const prov = el.closest("[data-province]");
  if (!prov) return undefined;
  const id = prov.getAttribute("data-province")!;
  return coast ? `${id}/${coast.getAttribute("data-coast")}` : id;
}

async function load(): Promise<Assets> {
  const [svg, layout, graphSvg, data] = await Promise.all([api.mapSvg(), api.mapLayout(), api.graphSvg(), api.map()]);

  const base = inlineSvg(svg);
  const map: BoardAsset = {
    markup: base.markup,
    viewBox: base.viewBox,
    scale: 1,
    position: (loc) => {
      if (loc.includes("/")) return layout.coasts[loc]?.unit ?? layout.provinces[loc.split("/")[0]!]?.unit;
      return layout.provinces[loc]?.unit;
    },
    hit: closestLocation,
  };

  // Graph view: node centres are read from the generated SVG (`node-Par`, `node-Spa-NC`).
  const g = inlineSvg(graphSvg);
  const nodes = new Map<string, Point>();
  for (const c of Array.from(g.doc.querySelectorAll("circle[id^='node-']"))) {
    const id = c.id.slice("node-".length).replace(/-(NC|SC|EC)$/, "/$1");
    nodes.set(id, [Number(c.getAttribute("cx")), Number(c.getAttribute("cy"))]);
  }
  const graph: BoardAsset = {
    markup: g.markup,
    viewBox: g.viewBox,
    scale: 2,
    unitOffset: [16, -14],
    position: (loc) => nodes.get(loc) ?? nodes.get(loc.split("/")[0]!),
    hit: (el) => {
      const node = el.closest("circle[id^='node-']");
      return node ? node.id.slice("node-".length).replace(/-(NC|SC|EC)$/, "/$1") : undefined;
    },
  };

  return { map, graph, data };
}

export function useAssets(): { assets?: Assets; error?: string } {
  const [state, setState] = useState<{ assets?: Assets; error?: string }>({});
  useEffect(() => {
    load().then(
      (assets) => setState({ assets }),
      (e: unknown) => setState({ error: String(e) }),
    );
  }, []);
  return state;
}
