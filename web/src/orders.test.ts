import { describe, expect, it } from "vitest";
import { orderText, parseTraceOrder } from "./orders";
import type { GameState } from "./types";

const state: GameState = {
  phase: "S1901M",
  units: [
    { power: "FRA", type: "A", location: "Par" },
    { power: "FRA", type: "A", location: "Mar" },
    { power: "ENG", type: "F", location: "Nth" },
    { power: "ENG", type: "A", location: "Lon" },
    { power: "RUS", type: "F", location: "StP", coast: "SC" },
  ],
  supplyCenters: {},
  dislodged: [],
  standoffProvinces: [],
  civilDisorder: [],
  winner: null,
};

describe("orderText (rules.md § 5 notation)", () => {
  it("writes moves, supports, convoys and builds", () => {
    expect(orderText(state, { power: "FRA", unit: "Par", type: "move", to: "Bur" })).toBe("A Par-Bur");
    expect(orderText(state, { power: "FRA", unit: "Mar", type: "support", target: "Par", to: "Bur" })).toBe("A Mar S A Par-Bur");
    expect(orderText(state, { power: "FRA", unit: "Mar", type: "support", target: "Par" })).toBe("A Mar S A Par");
    expect(orderText(state, { power: "ENG", unit: "Nth", type: "convoy", target: "Lon", to: "Nwy" })).toBe("F Nth C A Lon-Nwy");
    expect(orderText(state, { power: "RUS", unit: "StP", type: "move", to: "Bot" })).toBe("F StP/SC-Bot");
    expect(orderText(state, { power: "RUS", unit: "StP", type: "build", unitType: "F", coast: "NC" })).toBe("B F StP/NC");
  });
});

describe("parseTraceOrder (trace notation → arrows)", () => {
  it("parses moves, supports, convoys and holds", () => {
    expect(parseTraceOrder("A Mar-Bur")).toEqual({ kind: "move", from: "Mar", to: "Bur" });
    expect(parseTraceOrder("F StP/SC-Bot")).toEqual({ kind: "move", from: "StP/SC", to: "Bot" });
    expect(parseTraceOrder("A Par S A Mar-Bur")).toEqual({ kind: "support", from: "Par", target: "Mar", to: "Bur" });
    expect(parseTraceOrder("F Den S F Bal")).toEqual({ kind: "support", from: "Den", target: "Bal", to: undefined });
    expect(parseTraceOrder("F Nth C A Lon-Nwy")).toEqual({ kind: "convoy", from: "Nth", target: "Lon", to: "Nwy" });
    expect(parseTraceOrder("A Vie")).toEqual({ kind: "hold", at: "Vie" });
  });

  it("ignores text that is not an order", () => {
    expect(parseTraceOrder("Bur")).toBeUndefined();
    expect(parseTraceOrder("attack 2 > hold 0")).toBeUndefined();
  });
});
