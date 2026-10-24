import { describe, expect, it } from "vitest";
import type { GameState, Resolution } from "./types";
import { allArrows, boardAfter, buildUnitSteps, unitStepArrows } from "./unitSteps";

// Rulebook Diagram 8 plus a bystander: France A Mar-Bur supported by A Gas dislodges German A Bur;
// an Austrian army holds with no order; a Russian fleet on a split coast bounces.
const before: GameState = {
  phase: "S1901M",
  units: [
    { power: "GER", type: "A", location: "Bur" },
    { power: "FRA", type: "A", location: "Mar" },
    { power: "FRA", type: "A", location: "Gas" },
    { power: "AUS", type: "A", location: "Vie" },
    { power: "RUS", type: "F", location: "StP", coast: "SC" },
  ],
  supplyCenters: {},
  dislodged: [],
  standoffProvinces: [],
  civilDisorder: [],
  winner: null,
};

const resolution: Resolution = {
  phase: "S1901M",
  orders: [
    { order: { power: "FRA", unit: "Mar", type: "move", to: "Bur" }, valid: true, outcome: "success", rules: ["R-MOV-04"] },
    { order: { power: "FRA", unit: "Gas", type: "support", target: "Mar", to: "Bur" }, valid: true, outcome: "success", rules: [] },
    { order: { power: "GER", unit: "Bur", type: "hold" }, valid: true, outcome: "success", rules: [] },
    { order: { power: "RUS", unit: "StP", type: "move", to: "Bot" }, valid: true, outcome: "fail", reason: "standoff", rules: ["R-STD-01"] },
  ],
  trace: [
    { step: "validate", rules: [], order: "A Mar-Bur", outcome: "valid" },
    { step: "validate", rules: ["R-ORD-03"], outcome: "hold", units: ["A Vie"] },
    { step: "support-cuts", rules: [], order: "A Gas S A Mar-Bur", outcome: "given" },
    { step: "strengths", rules: [], province: "Bur", holder: "A Bur", moves: [{ order: "A Mar-Bur", attack: 2 }] },
    { step: "moves", rules: [], order: "A Mar-Bur", outcome: "success" },
    { step: "moves", rules: [], order: "F StP/SC-Bot", outcome: "fail" },
    { step: "result", rules: [], positions: ["FRA A Bur"] },
  ],
  nextState: {
    ...before,
    phase: "S1901R",
    units: [
      { power: "FRA", type: "A", location: "Bur" },
      { power: "FRA", type: "A", location: "Gas" },
      { power: "AUS", type: "A", location: "Vie" },
      { power: "RUS", type: "F", location: "StP", coast: "SC" },
    ],
    dislodged: [{ power: "GER", type: "A", location: "Bur", attackerFrom: "Mar", retreatOptions: ["Ruh", "Mun"] }],
  },
};

const steps = buildUnitSteps(before, resolution);

describe("buildUnitSteps", () => {
  it("orders units power by power (AUS … TUR)", () => {
    expect(steps.map((s) => s.label)).toEqual(["A Vie", "A Mar", "A Gas", "A Bur", "F StP/SC"]);
  });

  it("records each unit's order, outcome and final position", () => {
    const [vie, mar, gas, bur, stp] = steps;
    expect(vie!.result).toBeUndefined();
    expect(vie!.final).toBe("Vie");
    expect(mar!.final).toBe("Bur");
    expect(gas!.final).toBe("Gas");
    expect(bur!.final).toBeUndefined();
    expect(bur!.dislodged?.retreatOptions).toEqual(["Ruh", "Mun"]);
    expect(stp!.final).toBe("StP/SC");
  });

  it("collects the trace steps that mention the unit", () => {
    expect(steps[1]!.related.map((s) => s.step)).toEqual(["validate", "support-cuts", "strengths", "moves"]);
    expect(steps[0]!.related.map((s) => s.step)).toEqual(["validate"]);
    expect(steps[3]!.related.map((s) => s.step)).toEqual(["strengths"]);
  });
});

describe("boardAfter (cumulative)", () => {
  it("starts from the position before the phase", () => {
    expect(boardAfter(before, resolution, steps, 0).units).toEqual(before.units);
  });

  it("moves processed units and shows the unit they dislodged at once", () => {
    const b = boardAfter(before, resolution, steps, 2); // Vie, Mar done
    expect(b.units.find((u) => u.location === "Bur")).toEqual({ power: "FRA", type: "A", location: "Bur" });
    expect(b.units.filter((u) => u.location === "Bur")).toHaveLength(1);
    expect(b.dislodged.map((d) => d.location)).toEqual(["Bur"]);
  });

  it("ends with the resolved position", () => {
    expect(boardAfter(before, resolution, steps, steps.length)).toBe(resolution.nextState);
  });
});

describe("arrows", () => {
  it("draws the unit's order and, muted, the supports it received", () => {
    expect(unitStepArrows(steps[1], resolution)).toEqual([
      { kind: "move", from: "Mar", to: "Bur", power: "FRA", status: "ok" },
      { kind: "support", from: "Gas", target: "Mar", to: "Bur", power: "FRA", status: "muted" },
    ]);
    expect(unitStepArrows(steps[4], resolution)[0]).toMatchObject({ kind: "move", status: "bad" });
    expect(unitStepArrows(steps[0], resolution)).toEqual([{ kind: "hold", at: "Vie", power: "AUS", status: "ok" }]);
  });

  it("draws every non-hold order for the before/after views", () => {
    expect(allArrows(resolution)).toHaveLength(3);
  });
});
