// Map drawing assets: docs/en/map.md § 7 (base map A, graph view D).
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { standardLayout, standardMap, unitPosition } from "./index";

const read = (name: string) => readFileSync(fileURLToPath(new URL(name, import.meta.url)));
const mapSvg = read("standard.svg").toString();
const graphSvg = read("standard.graph.svg").toString();

const { provinces, adjacency } = standardMap;
const splitCoasts = provinces.flatMap((p) => (p.coasts ?? []).map((c) => `${p.id}/${c}`));
const [vx, vy, vw, vh] = standardLayout.viewBox;
const inView = ([x, y]: [number, number]) => x >= vx && x <= vx + vw && y >= vy && y <= vy + vh;

describe("layout (standard.layout.json)", () => {
  it("has a unit and label position for every province, inside the view box", () => {
    expect(Object.keys(standardLayout.provinces).sort()).toEqual(provinces.map((p) => p.id).sort());
    for (const p of provinces) {
      const entry = standardLayout.provinces[p.id]!;
      expect(inView(entry.unit), p.id).toBe(true);
      expect(inView(entry.label), p.id).toBe(true);
    }
  });

  it("has an SC marker position exactly for supply centers", () => {
    for (const p of provinces) {
      expect(standardLayout.provinces[p.id]!.sc !== null, p.id).toBe(p.supplyCenter);
    }
  });

  it("has a fleet position for every split coast (R-MAP-06)", () => {
    expect(Object.keys(standardLayout.coasts).sort()).toEqual(splitCoasts.sort());
    for (const loc of splitCoasts) expect(inView(unitPosition(loc)!), loc).toBe(true);
  });

  it("matches the pinned source file", () => {
    const digest = createHash("sha256").update(read(standardLayout.source.file)).digest("hex");
    expect(digest).toBe(standardLayout.source.sha256);
  });
});

describe("base map (standard.svg)", () => {
  it("has one province group per province, plus impassable Switzerland", () => {
    const ids = [...mapSvg.matchAll(/<g id="prov-([A-Za-z]+)"/g)].map((m) => m[1]);
    expect(ids.sort()).toEqual([...provinces.map((p) => p.id), "Swi"].sort());
    for (const p of provinces) {
      expect(mapSvg, p.id).toContain(`id="prov-${p.id}" class="province ${p.type}" data-province="${p.id}"`);
    }
  });

  it("has a separately addressable area for each split coast", () => {
    const ids = [...mapSvg.matchAll(/id="coast-([A-Za-z]+)-([A-Z]{2})"/g)].map((m) => `${m[1]}/${m[2]}`);
    expect(ids.sort()).toEqual(splitCoasts.sort());
  });

  it("has SC markers exactly on supply centers", () => {
    const ids = [...mapSvg.matchAll(/id="sc-([A-Za-z]+)"/g)].map((m) => m[1]);
    expect(ids.sort()).toEqual(provinces.filter((p) => p.supplyCenter).map((p) => p.id).sort());
  });

  it("credits the source and states the licence (CC BY-SA 3.0)", () => {
    expect(mapSvg).toContain("Martin Asal");
    expect(mapSvg).toContain("https://commons.wikimedia.org/wiki/File:Diplomacy.svg");
    expect(mapSvg).toContain("CC BY-SA 3.0");
  });
});

describe("graph view (standard.graph.svg)", () => {
  it("has a node for every province and split coast", () => {
    for (const p of provinces) expect(graphSvg, p.id).toContain(`id="node-${p.id}"`);
    for (const loc of splitCoasts) expect(graphSvg, loc).toContain(`id="node-${loc.replace("/", "-")}"`);
  });

  it("has one edge per adjacent pair, typed army / fleet / both", () => {
    const kinds = new Map<string, Set<string>>();
    for (const [kind, table] of [["army", adjacency.army], ["fleet", adjacency.fleet]] as const) {
      for (const [a, bs] of Object.entries(table)) {
        for (const b of bs) {
          const key = [a, b].sort().join("|");
          kinds.set(key, (kinds.get(key) ?? new Set()).add(kind));
        }
      }
    }
    const edges = [...graphSvg.matchAll(/class="edge (\w+)" data-from="([^"]+)" data-to="([^"]+)"/g)];
    expect(edges).toHaveLength(kinds.size);
    for (const [, cls, a, b] of edges) {
      const k = kinds.get([a!, b!].sort().join("|"));
      expect(k, `${a}-${b}`).toBeDefined();
      expect(cls).toBe(k!.size === 2 ? "both" : [...k!][0]);
    }
  });
});
