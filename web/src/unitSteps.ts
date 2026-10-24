// Movement resolution, one unit at a time: the default way the GUI replays a movement phase.
// Built from the server's `Resolution` (orders with outcomes, trace, next position); no extra API.
import { orderArrow, province, type Arrow } from "./orders";
import type { DislodgedUnit, GameState, Order, OrderResult, Resolution, TraceStep, Unit } from "./types";
import { POWERS } from "./types";

export type ViewMode = "units" | "before" | "after";

export type StatusArrow = Arrow & { status?: "ok" | "bad" | "muted" };

export interface UnitStep {
  unit: Unit;
  /** Trace notation of the unit, e.g. `A Par`, `F StP/SC`. */
  label: string;
  /** The order the unit was given (absent: no order, the unit holds — R-ORD-03). */
  result?: OrderResult;
  /** Where the unit ends up (`Bur`, `Spa/NC`); undefined if dislodged. */
  final?: string;
  dislodged?: DislodgedUnit;
  /** Trace steps that mention this unit (validation, support cuts, strengths, moves, …). */
  related: TraceStep[];
}

const labelOf = (u: { type: string; location: string; coast?: string }) =>
  `${u.type} ${u.location}${u.coast ? `/${u.coast}` : ""}`;

/**
 * Does any string in `v` name the unit `label`, as the ordered unit (`A Par-Bur`) or as the unit
 * supported or convoyed (`A Gas S A Par-Bur`)?
 */
function mentions(v: unknown, label: string): boolean {
  if (typeof v === "string") return (` ${v} `).replace(/-/g, " - ").includes(` ${label} `);
  if (Array.isArray(v)) return v.some((x) => mentions(x, label));
  if (v && typeof v === "object") return Object.values(v).some((x) => mentions(x, label));
  return false;
}

/** One step per unit of the starting position, power by power (AUS … TUR), in board order. */
export function buildUnitSteps(before: GameState, resolution: Resolution): UnitStep[] {
  const next = resolution.nextState;
  const steps: UnitStep[] = [];
  for (const power of POWERS) {
    for (const unit of before.units.filter((u) => u.power === power)) {
      const label = labelOf(unit);
      const result = resolution.orders.find((r) => r.order.power === power && r.order.unit === unit.location);
      const dislodged = next.dislodged.find((d) => d.power === power && d.location === unit.location);
      let final: string | undefined;
      if (!dislodged) {
        const moved = result?.order.type === "move" && result.outcome === "success" ? result.order.to : undefined;
        const at = moved ? province(moved) : unit.location;
        const there = next.units.find((u) => u.power === power && u.location === at && u.type === unit.type);
        final = there?.coast ? `${at}/${there.coast}` : (moved ?? labelOf(unit).slice(2));
      }
      const related = resolution.trace.filter((s) => s.step !== "result" && mentions(s, label));
      steps.push({ unit, label, result, final, dislodged, related });
    }
  }
  return steps;
}

/**
 * The board after the first `count` unit steps: those units are at their final positions (or
 * shown dislodged), the others where they started. A unit dislodged by an attacker that has
 * already been shown is shown dislodged too, so two units never share a province for that reason.
 */
export function boardAfter(before: GameState, resolution: Resolution, steps: UnitStep[], count: number): GameState {
  if (count <= 0) return before;
  if (count >= steps.length) return resolution.nextState;
  const done = steps.slice(0, count);
  const arrived = new Set(done.flatMap((s) => (s.final && s.final !== s.label.slice(2) ? [province(s.final)] : [])));
  const units: Unit[] = [];
  const dislodged: DislodgedUnit[] = [];
  steps.forEach((s, i) => {
    const processed = i < count;
    if (s.dislodged && (processed || arrived.has(s.unit.location))) {
      dislodged.push(s.dislodged);
      return;
    }
    if (processed && s.final) {
      const [loc, coast] = s.final.split("/");
      units.push({ power: s.unit.power, type: s.unit.type, location: loc!, ...(coast ? { coast } : {}) });
    } else {
      units.push(s.unit);
    }
  });
  return { ...before, units, dislodged };
}

const statusOf = (r: OrderResult | undefined): "ok" | "bad" => (!r || r.outcome === "success" ? "ok" : "bad");

/** Arrows for one unit step: its own order, plus (muted) the supports and convoys given to it. */
export function unitStepArrows(step: UnitStep | undefined, resolution: Resolution): StatusArrow[] {
  if (!step) return [];
  const own: Order = step.result?.order ?? { power: step.unit.power, unit: step.unit.location, type: "hold" };
  const arrows: StatusArrow[] = [{ ...orderArrow(own), status: statusOf(step.result) }];
  for (const r of resolution.orders) {
    if ((r.order.type === "support" || r.order.type === "convoy") && r.order.target === step.unit.location) {
      arrows.push({ ...orderArrow(r.order), status: "muted" });
    }
  }
  return arrows;
}

/** Arrows for every order of the phase, coloured by outcome (the before/after views). */
export function allArrows(resolution: Resolution): StatusArrow[] {
  return resolution.orders
    .filter((r) => r.order.type !== "hold")
    .map((r) => ({ ...orderArrow(r.order), status: statusOf(r) }));
}
