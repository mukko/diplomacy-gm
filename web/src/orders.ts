// Order notation (rules.md § 5) and the arrows drawn for orders on the map.
import type { GameState, Order, PowerId, Unit } from "./types";

export const POWER_COLORS: Record<PowerId, string> = {
  AUS: "#c0392b",
  ENG: "#1f3a93",
  FRA: "#4aa3df",
  GER: "#3d3d3d",
  ITA: "#27ae60",
  RUS: "#f4f4f4",
  TUR: "#f1c40f",
};

/** Light tint of a power colour for owned territory. */
export const POWER_TINTS: Record<PowerId, string> = {
  AUS: "#f2c4bf",
  ENG: "#b9c4e6",
  FRA: "#c4e3f6",
  GER: "#c9c9c9",
  ITA: "#bfe8cf",
  RUS: "#ffffff",
  TUR: "#fbeba8",
};

export const province = (location: string) => location.split("/")[0]!;

/** `"StP/SC"` for a fleet on a split coast, otherwise the province. */
export const unitLocation = (u: Unit) => (u.coast ? `${u.location}/${u.coast}` : u.location);

export function unitAt(state: GameState, prov: string): Unit | undefined {
  return state.units.find((u) => u.location === province(prov));
}

function unitLabel(state: GameState, prov: string): string {
  const u = unitAt(state, prov) ?? state.dislodged.find((d) => d.location === province(prov));
  return u ? `${u.type} ${unitLocation(u)}` : prov;
}

/** Rulebook notation of an order, e.g. `A Par S A Mar-Bur`, `F Mid C A Lon-Bre`, `B F StP/NC`. */
export function orderText(state: GameState, o: Order): string {
  switch (o.type) {
    case "hold":
      return `${unitLabel(state, o.unit)} H`;
    case "move":
      return `${unitLabel(state, o.unit)}-${o.to}${o.viaConvoy ? " (via convoy)" : ""}`;
    case "support":
      return `${unitLabel(state, o.unit)} S ${unitLabel(state, o.target)}${o.to ? `-${o.to}` : ""}`;
    case "convoy":
      return `${unitLabel(state, o.unit)} C ${unitLabel(state, o.target)}-${o.to}`;
    case "retreat":
      return `${unitLabel(state, o.unit)}-${o.to}`;
    case "disband":
      return `${unitLabel(state, o.unit)} D`;
    case "build":
      return `B ${o.unitType} ${o.unit}${o.coast ? `/${o.coast}` : ""}`;
    case "waive":
      return `W ${o.unit}`;
  }
}

export const sameOrder = (a: Order, b: Order) => JSON.stringify(a) === JSON.stringify(b);

/** Arrow to draw on the map. Locations are provinces or `Prov/Coast`. */
export type Arrow =
  | { kind: "move" | "retreat"; from: string; to: string; power?: PowerId }
  | { kind: "support"; from: string; target: string; to?: string; power?: PowerId }
  | { kind: "convoy"; from: string; target: string; to: string; power?: PowerId }
  | { kind: "hold" | "disband" | "build"; at: string; power?: PowerId };

export function orderArrow(o: Order): Arrow {
  switch (o.type) {
    case "move":
    case "retreat":
      return { kind: o.type, from: o.unit, to: o.to, power: o.power };
    case "support":
      return { kind: "support", from: o.unit, target: o.target, to: o.to, power: o.power };
    case "convoy":
      return { kind: "convoy", from: o.unit, target: o.target, to: o.to, power: o.power };
    case "build":
      return { kind: "build", at: o.unit, power: o.power };
    case "disband":
      return { kind: "disband", at: o.unit, power: o.power };
    default:
      return { kind: "hold", at: o.unit, power: o.power };
  }
}

/**
 * Parse the notation used in resolution traces (`A Mar-Bur`, `A Par S A Mar-Bur`,
 * `F Nth C A Lon-Nwy`, `A Vie`, `F StP/SC`) into an arrow. Unknown text → undefined.
 */
export function parseTraceOrder(text: string): Arrow | undefined {
  const loc = String.raw`([A-Za-z]{3}(?:/[NSE]C)?)`;
  const u = String.raw`(?:[AF] )?`;
  let m = new RegExp(`^B [AF] ${loc}$`).exec(text);
  if (m) return { kind: "build", at: m[1]! };
  m = new RegExp(`^[AF] ${loc} D$`).exec(text);
  if (m) return { kind: "disband", at: m[1]! };
  m = new RegExp(`^${u}${loc} [SC] ${u}${loc}(?:-${loc})?`).exec(text);
  if (m) {
    const kind = / C /.test(text) ? "convoy" : "support";
    return kind === "convoy"
      ? { kind, from: m[1]!, target: m[2]!, to: m[3] ?? m[2]! }
      : { kind, from: m[1]!, target: m[2]!, to: m[3] };
  }
  m = new RegExp(`^${u}${loc}-${loc}`).exec(text);
  if (m) return { kind: "move", from: m[1]!, to: m[2]! };
  m = new RegExp(`^[AF] ${loc}(?: H)?$`).exec(text);
  if (m) return { kind: "hold", at: m[1]! };
  return undefined;
}
