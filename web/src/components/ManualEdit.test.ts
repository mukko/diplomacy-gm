import { describe, expect, it } from "vitest";
import type { GameState, MapData } from "../types";
import { allowedTypes, clickDraft, draftChanges, startDraft } from "./ManualEdit";

const map: MapData = {
  provinces: [
    { id: "Par", name: "Paris", type: "land", supplyCenter: true, home: "FRA" },
    { id: "Bur", name: "Burgundy", type: "land", supplyCenter: false, home: null },
    { id: "Mid", name: "Mid-Atlantic Ocean", type: "sea", supplyCenter: false, home: null },
    { id: "Spa", name: "Spain", type: "coast", supplyCenter: true, home: null, coasts: ["NC", "SC"] },
    { id: "Bre", name: "Brest", type: "coast", supplyCenter: true, home: "FRA" },
  ],
};
const state: GameState = {
  phase: "S1901M",
  units: [
    { power: "FRA", type: "A", location: "Par" },
    { power: "FRA", type: "F", location: "Bre" },
  ],
  supplyCenters: { Par: "FRA", Bre: "FRA", Spa: null },
  dislodged: [],
  standoffProvinces: [],
  civilDisorder: [],
  winner: null,
};

describe("manual edit draft", () => {
  it("allows unit types by province type (R-MAP-02)", () => {
    expect(allowedTypes(map.provinces[0]!)).toEqual(["A"]);
    expect(allowedTypes(map.provinces[2]!)).toEqual(["F"]);
    expect(allowedTypes(map.provinces[3]!)).toEqual(["A", "F"]);
  });

  it("moves a unit to an empty province it may stand in", () => {
    let d = clickDraft(startDraft(state), "Par", map);
    d = clickDraft({ ...d, moving: "Par" }, "Bur", map);
    expect(d.units.find((u) => u.location === "Bur")).toEqual({ power: "FRA", type: "A", location: "Bur" });
    expect(d.moving).toBeUndefined();
    expect(draftChanges(state, d)).toEqual(["− FRA A Par", "+ FRA A Bur"]);
  });

  it("does not move an army to sea or onto another unit", () => {
    const d0 = { ...startDraft(state), moving: "Par" };
    expect(clickDraft(d0, "Mid", map).units).toEqual(state.units);
    expect(clickDraft(d0, "Bre", map).units).toEqual(state.units);
  });

  it("puts a fleet moved to a split-coast province on a coast", () => {
    const d = clickDraft({ ...startDraft(state), moving: "Bre" }, "Spa", map);
    expect(d.units.find((u) => u.location === "Spa")).toEqual({ power: "FRA", type: "F", location: "Spa", coast: "NC" });
  });

  it("lists supply-center owner changes", () => {
    const d = { ...startDraft(state), supplyCenters: { ...state.supplyCenters, Spa: "FRA" as const } };
    expect(draftChanges(state, d)).toEqual(["Spa: — → FRA"]);
  });
});
