// Map data tests: docs/en/map.md §1/§3/§4 and docs/en/test-cases.md §1 (RB-D01..D03, RB-SPA, RB-BUL).
import { describe, expect, it } from "vitest";
import {
  adjacentLocations,
  canMove,
  getProvince,
  parseLocation,
  resolveFleetDestination,
  standardMap,
} from "./index";

const { provinces, adjacency } = standardMap;
const sorted = (xs: string[]) => [...xs].sort();

describe("totals (map.md §1, R-MAP-03)", () => {
  const count = (pred: (p: (typeof provinces)[number]) => boolean) => provinces.filter(pred).length;

  it("has 75 provinces with unique IDs", () => {
    expect(provinces).toHaveLength(75);
    expect(new Set(provinces.map((p) => p.id)).size).toBe(75);
  });

  it("has 14 inland, 42 coastal and 19 water provinces", () => {
    expect(count((p) => p.type === "land")).toBe(14);
    expect(count((p) => p.type === "coast")).toBe(42);
    expect(count((p) => p.type === "sea")).toBe(19);
  });

  it("has 34 supply centers, 22 home and 12 neutral", () => {
    expect(count((p) => p.supplyCenter)).toBe(34);
    expect(count((p) => p.home !== null)).toBe(22);
    expect(count((p) => p.supplyCenter && p.home === null)).toBe(12);
  });

  it("only supply centers on land have a home power", () => {
    for (const p of provinces) {
      if (p.home !== null) expect(p.supplyCenter, p.id).toBe(true);
      if (p.type === "sea") expect(p.supplyCenter, p.id).toBe(false);
    }
  });

  it("gives each power the rulebook number of home centers (Russia 4, others 3)", () => {
    const homes: Record<string, string[]> = {};
    for (const p of provinces) if (p.home) (homes[p.home] ??= []).push(p.id);
    expect(Object.fromEntries(Object.entries(homes).map(([k, v]) => [k, sorted(v)]))).toEqual({
      AUS: ["Bud", "Tri", "Vie"],
      ENG: ["Edi", "Lon", "Lvp"],
      FRA: ["Bre", "Mar", "Par"],
      GER: ["Ber", "Kie", "Mun"],
      ITA: ["Nap", "Rom", "Ven"],
      RUS: ["Mos", "Sev", "StP", "War"],
      TUR: ["Ank", "Con", "Smy"],
    });
  });
});

describe("split coasts (map.md §3, R-MAP-06)", () => {
  it("exactly Bul (EC/SC), Spa (NC/SC) and StP (NC/SC) have coasts", () => {
    const split = provinces.filter((p) => p.coasts).map((p) => [p.id, p.coasts]);
    expect(split).toEqual(
      expect.arrayContaining([
        ["Bul", ["EC", "SC"]],
        ["Spa", ["NC", "SC"]],
        ["StP", ["NC", "SC"]],
      ]),
    );
    expect(split).toHaveLength(3);
  });

  it("fleet adjacency uses coast IDs for split-coast provinces, never the bare province", () => {
    for (const p of provinces.filter((p) => p.coasts)) {
      expect(adjacency.fleet[p.id], p.id).toBeUndefined();
      for (const c of p.coasts!) expect(adjacency.fleet[`${p.id}/${c}`], `${p.id}/${c}`).toBeDefined();
      for (const targets of Object.values(adjacency.fleet)) expect(targets).not.toContain(p.id);
    }
  });
});

describe("adjacency structure", () => {
  const isValidFleetLocation = (loc: string) => {
    const { province, coast } = parseLocation(loc);
    const p = getProvince(province);
    if (!p || p.type === "land") return false;
    return p.coasts ? coast !== undefined && p.coasts.includes(coast) : coast === undefined;
  };

  it("army adjacency covers exactly the 56 land/coastal provinces, never water (R-MOV-01)", () => {
    const landIds = provinces.filter((p) => p.type !== "sea").map((p) => p.id);
    expect(sorted(Object.keys(adjacency.army))).toEqual(sorted(landIds));
    for (const [from, targets] of Object.entries(adjacency.army)) {
      for (const to of targets) expect(getProvince(to)?.type, `${from}-${to}`).not.toBe("sea");
    }
  });

  it("fleet adjacency covers every water and coastal location, never inland (R-MOV-02)", () => {
    const expected = provinces
      .filter((p) => p.type !== "land")
      .flatMap((p) => (p.coasts ? p.coasts.map((c) => `${p.id}/${c}`) : [p.id]));
    expect(sorted(Object.keys(adjacency.fleet))).toEqual(sorted(expected));
    for (const [from, targets] of Object.entries(adjacency.fleet)) {
      for (const to of targets) expect(isValidFleetLocation(to), `${from}-${to}`).toBe(true);
    }
  });

  it("has no self-loops or duplicate entries", () => {
    for (const table of [adjacency.army, adjacency.fleet]) {
      for (const [from, targets] of Object.entries(table)) {
        expect(targets, from).not.toContain(from);
        expect(new Set(targets).size, from).toBe(targets.length);
      }
    }
  });

  it("army adjacency is symmetric", () => {
    for (const [from, targets] of Object.entries(adjacency.army)) {
      for (const to of targets) expect(adjacency.army[to], `${to} should list ${from}`).toContain(from);
    }
  });

  it("fleet adjacency is symmetric (per coast)", () => {
    for (const [from, targets] of Object.entries(adjacency.fleet)) {
      for (const to of targets) expect(adjacency.fleet[to], `${to} should list ${from}`).toContain(from);
    }
  });

  it("a fleet move between two land provinces implies they share a land border", () => {
    for (const [from, targets] of Object.entries(adjacency.fleet)) {
      const a = parseLocation(from).province;
      for (const to of targets) {
        const b = parseLocation(to).province;
        if (getProvince(a)?.type === "coast" && getProvince(b)?.type === "coast") {
          expect(adjacency.army[a], `${a}-${b}`).toContain(b);
        }
      }
    }
  });

  it("every coastal province borders at least one water province (R-MAP-01)", () => {
    for (const p of provinces.filter((p) => p.type === "coast")) {
      const locs = p.coasts ? p.coasts.map((c) => `${p.id}/${c}`) : [p.id];
      const seas = locs.flatMap((l) => adjacency.fleet[l] ?? []).filter((t) => getProvince(t)?.type === "sea");
      expect(seas.length, p.id).toBeGreaterThan(0);
    }
  });

  it("each split coast borders at least one water province", () => {
    for (const [loc, targets] of Object.entries(adjacency.fleet)) {
      if (!loc.includes("/")) continue;
      expect(targets.some((t) => getProvince(t)?.type === "sea"), loc).toBe(true);
    }
  });
});

describe("special map cases (map.md §4)", () => {
  it("Switzerland is impassable and not a province (R-MAP-04)", () => {
    expect(getProvince("Swi")).toBeUndefined();
    for (const table of [adjacency.army, adjacency.fleet]) {
      for (const targets of Object.values(table)) expect(targets).not.toContain("Swi");
    }
  });

  it("Kiel has one coast: F Hol-Kie and F Kie-Ber/Bal/Hel/Den (R-MAP-08)", () => {
    expect(getProvince("Kie")?.coasts).toBeUndefined();
    expect(sorted(adjacentLocations("F", "Kie"))).toEqual(["Bal", "Ber", "Den", "Hel", "Hol"]);
    expect(canMove("A", "Kie", "Ber")).toBe(true);
    expect(canMove("A", "Kie", "Hol")).toBe(true);
  });

  it("Constantinople has one coast: F Bla-Con and F Con-Aeg/Bul/Smy/Ank (R-MAP-08)", () => {
    expect(getProvince("Con")?.coasts).toBeUndefined();
    expect(sorted(adjacentLocations("F", "Con"))).toEqual(["Aeg", "Ank", "Bla", "Bul/EC", "Bul/SC", "Smy"]);
    expect(canMove("F", "Bla", "Con")).toBe(true);
    expect(canMove("F", "Aeg", "Con")).toBe(true);
  });

  it("Denmark and Sweden are directly adjacent for armies and fleets (R-MAP-09)", () => {
    expect(canMove("A", "Den", "Swe")).toBe(true);
    expect(canMove("A", "Swe", "Den")).toBe(true);
    expect(canMove("F", "Den", "Swe")).toBe(true);
    expect(canMove("F", "Swe", "Den")).toBe(true);
  });

  it("Baltic Sea and Skagerrak are not adjacent (R-MAP-09)", () => {
    expect(canMove("F", "Bal", "Ska")).toBe(false);
    expect(canMove("F", "Ska", "Bal")).toBe(false);
  });

  it("Denmark does not border Berlin (R-MAP-09)", () => {
    expect(canMove("A", "Den", "Ber")).toBe(false);
    expect(canMove("F", "Den", "Ber")).toBe(false);
    expect(canMove("A", "Ber", "Den")).toBe(false);
    expect(canMove("F", "Ber", "Den")).toBe(false);
  });

  it("Sweden and Norway share a coastline (R-MAP-05)", () => {
    expect(canMove("F", "Swe", "Nwy")).toBe(true);
    expect(canMove("F", "Nwy", "Swe")).toBe(true);
    expect(canMove("A", "Swe", "Nwy")).toBe(true);
  });

  it("Spain NC is not adjacent to Wes, GoL or Mar; Spain SC is (R-MAP-06)", () => {
    for (const to of ["Wes", "GoL", "Mar"]) {
      expect(canMove("F", "Spa/NC", to), to).toBe(false);
      expect(canMove("F", "Spa/SC", to), to).toBe(true);
    }
  });
});

describe("rulebook examples, movement / map (test-cases.md §1)", () => {
  it("RB-D01: A Par may move to Bre, Pic, Bur, Gas (R-MOV-01)", () => {
    expect(sorted(adjacentLocations("A", "Par"))).toEqual(["Bre", "Bur", "Gas", "Pic"]);
  });

  it("RB-D02: F Eng may move to Iri, Wal, Lon, Bel, Pic, Bre, Nth, Mid (R-MOV-02)", () => {
    expect(sorted(adjacentLocations("F", "Eng"))).toEqual(sorted(["Iri", "Wal", "Lon", "Bel", "Pic", "Bre", "Nth", "Mid"]));
  });

  it("RB-D03: F Rom may move to Tus, Nap, Tyn but not Ven, Apu (R-MAP-05)", () => {
    expect(sorted(adjacentLocations("F", "Rom"))).toEqual(["Nap", "Tus", "Tyn"]);
    expect(canMove("F", "Rom", "Ven")).toBe(false);
    expect(canMove("F", "Rom", "Apu")).toBe(false);
    // An army in Rome does border Venice and Apulia.
    expect(canMove("A", "Rom", "Ven")).toBe(true);
    expect(canMove("A", "Rom", "Apu")).toBe(true);
  });

  it("RB-SPA: F Spa/NC to Wes, GoL, Mar are all illegal (R-MAP-06)", () => {
    for (const to of ["Wes", "GoL", "Mar"]) {
      expect(resolveFleetDestination("Spa/NC", to), to).toEqual({ ok: false, reason: "not-adjacent" });
    }
  });

  it("RB-BUL: F Con-Bul without coast is illegal; with EC or SC it is legal (R-MAP-07)", () => {
    expect(resolveFleetDestination("Con", "Bul")).toEqual({ ok: false, reason: "coast-required" });
    expect(resolveFleetDestination("Con", "Bul/EC")).toEqual({ ok: true, location: "Bul/EC" });
    expect(resolveFleetDestination("Con", "Bul/SC")).toEqual({ ok: true, location: "Bul/SC" });
  });

  it("PR-04: coast is inferred when only one is reachable, and an unreachable coast is invalid", () => {
    expect(resolveFleetDestination("Bla", "Bul")).toEqual({ ok: true, location: "Bul/EC" });
    expect(resolveFleetDestination("Bla", "Bul/SC")).toEqual({ ok: false, reason: "coast-unreachable" });
    expect(resolveFleetDestination("Gas", "Spa")).toEqual({ ok: true, location: "Spa/NC" });
    expect(resolveFleetDestination("Mid", "Spa")).toEqual({ ok: false, reason: "coast-required" });
    expect(resolveFleetDestination("Bot", "StP")).toEqual({ ok: true, location: "StP/SC" });
  });
});

describe("starting fleets (map.md §5)", () => {
  it("Russia's fleet at StP/SC can reach Bot (sample game F StP-Bot)", () => {
    expect(canMove("F", "StP/SC", "Bot")).toBe(true);
    expect(canMove("F", "StP/NC", "Bot")).toBe(false);
  });
});
