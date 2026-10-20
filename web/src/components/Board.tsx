// Board drawing shared by the map view (standard.svg) and the graph view (standard.graph.svg):
// the static SVG is inlined, and units, ownership, selection and order arrows are drawn on top.
import { useMemo, type MouseEvent } from "react";
import { POWER_COLORS, POWER_TINTS, province, unitLocation, type Arrow } from "../orders";
import type { GameState, Point, PowerId } from "../types";

export interface BoardAsset {
  /** Inner markup of the source <svg>, with its stylesheet scoped to `.board`. */
  markup: string;
  viewBox: string;
  /** Position of a unit at a location (`Par`, `Spa/NC`). */
  position: (location: string) => Point | undefined;
  /** Unit and arrow size relative to the map view. */
  scale: number;
  /** Where units sit relative to their position (the graph view keeps node labels visible). */
  unitOffset?: Point;
  /** Which province / coast a click landed on. */
  hit: (el: Element) => string | undefined;
}

/** Inline an SVG document: returns its inner markup with `<style>` selectors prefixed by `.board`. */
export function inlineSvg(text: string): { markup: string; viewBox: string; doc: Document } {
  const doc = new DOMParser().parseFromString(text, "image/svg+xml");
  const root = doc.documentElement;
  for (const style of Array.from(root.querySelectorAll("style"))) {
    style.textContent = (style.textContent ?? "").replace(/([^{}]+)\{/g, (_, sel: string) =>
      sel
        .split(",")
        .map((s) => `.board ${s.trim()}`)
        .join(", ") + " {",
    );
  }
  return { markup: root.innerHTML, viewBox: root.getAttribute("viewBox") ?? "0 0 100 100", doc };
}

interface Props {
  asset: BoardAsset;
  state: GameState;
  selected?: string;
  /** Provinces offered as targets for the selected unit. */
  targets?: string[];
  arrows?: (Arrow & { status?: "ok" | "bad" | "muted" })[];
  /** Units to emphasise (e.g. dislodged in the current resolution step). */
  marked?: string[];
  onSelect?: (location: string) => void;
}

export function Board({ asset, state, selected, targets = [], arrows = [], marked = [], onSelect }: Props) {
  const s = asset.scale;
  const css = useMemo(() => {
    const rules: string[] = [];
    for (const [prov, owner] of Object.entries(state.supplyCenters)) {
      if (!owner) continue;
      rules.push(`.board #prov-${prov} .area { fill: ${POWER_TINTS[owner]}; }`);
      rules.push(`.board #sc-${prov} { fill: ${POWER_COLORS[owner]}; }`);
      rules.push(`.board #node-${prov} { stroke: ${POWER_COLORS[owner] === "#f4f4f4" ? "#888" : POWER_COLORS[owner]}; }`);
    }
    for (const t of targets) {
      rules.push(`.board #prov-${province(t)} .area { fill: #fff1a8; }`);
      rules.push(`.board #node-${t.replace("/", "-")} { fill: #fff1a8; }`);
    }
    if (selected) {
      rules.push(`.board #prov-${province(selected)} .area { stroke: #e67e22; stroke-width: 2.5; }`);
      rules.push(`.board #node-${province(selected)} { stroke: #e67e22; stroke-width: 3; }`);
    }
    return rules.join("\n");
  }, [state.supplyCenters, targets, selected]);

  const click = (e: MouseEvent<SVGSVGElement>) => {
    const loc = asset.hit(e.target as Element);
    if (loc && onSelect) onSelect(loc);
  };

  return (
    <svg className="board" viewBox={asset.viewBox} onClick={click} role="img">
      <style>{css}</style>
      <g dangerouslySetInnerHTML={{ __html: asset.markup }} />
      <defs>
        {(["ok", "bad", "muted", "default"] as const).map((k) => (
          <marker key={k} id={`arrow-${k}`} viewBox="0 0 10 10" refX="8" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
            <path d="M0,0 L10,5 L0,10 z" className={`arrowhead ${k}`} />
          </marker>
        ))}
      </defs>
      <g className="arrows" pointerEvents="none">
        {arrows.map((a, i) => (
          <ArrowShape key={i} arrow={a} asset={asset} />
        ))}
      </g>
      <g className="units" pointerEvents="none">
        {state.units.map((u) => {
          const loc = unitLocation(u);
          const p = asset.position(loc);
          const [ox, oy] = asset.unitOffset ?? [0, 0];
          return p ? <UnitShape key={loc} type={u.type} power={u.power} at={[p[0] + ox, p[1] + oy]} scale={s} marked={marked.includes(u.location)} /> : null;
        })}
        {state.dislodged.map((u) => {
          const p = asset.position(unitLocation(u));
          return p ? (
            <UnitShape key={`d-${u.location}`} type={u.type} power={u.power} at={[p[0] + 7 * s, p[1] + 7 * s]} scale={s} dislodged />
          ) : null;
        })}
      </g>
    </svg>
  );
}

function UnitShape({ type, power, at, scale, dislodged, marked }: { type: "A" | "F"; power: PowerId; at: Point; scale: number; dislodged?: boolean; marked?: boolean }) {
  const [x, y] = at;
  const fill = POWER_COLORS[power];
  const dark = power === "RUS" || power === "TUR" || power === "FRA";
  const cls = `unit${dislodged ? " dislodged" : ""}${marked ? " marked" : ""}`;
  return (
    <g className={cls} transform={`translate(${x},${y}) scale(${scale})`}>
      {type === "A" ? <circle r="6.5" fill={fill} /> : <rect x="-8" y="-5.5" width="16" height="11" rx="3" fill={fill} />}
      <text y="0.5" fill={dark ? "#000" : "#fff"}>
        {type}
      </text>
    </g>
  );
}

function ArrowShape({ arrow, asset }: { arrow: Arrow & { status?: string }; asset: BoardAsset }) {
  const pos = asset.position;
  const cls = `arrow ${arrow.kind} ${arrow.status ?? "default"}`;
  const marker = `url(#arrow-${arrow.status ?? "default"})`;
  const shorten = (a: Point, b: Point, by: number): Point => {
    const dx = b[0] - a[0];
    const dy = b[1] - a[1];
    const n = Math.hypot(dx, dy) || 1;
    return [b[0] - (dx / n) * by, b[1] - (dy / n) * by];
  };
  const mid = (a: Point, b: Point): Point => [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];
  const line = (a: Point, b: Point, end = 7) => {
    const e = shorten(a, b, end * asset.scale);
    return <line className={cls} x1={a[0]} y1={a[1]} x2={e[0]} y2={e[1]} markerEnd={marker} />;
  };

  switch (arrow.kind) {
    case "move":
    case "retreat": {
      const a = pos(arrow.from);
      const b = pos(arrow.to);
      return a && b ? line(a, b) : null;
    }
    case "support": {
      const a = pos(arrow.from);
      const t = pos(arrow.target);
      if (!a || !t) return null;
      if (!arrow.to) return line(a, t, 9);
      const d = pos(arrow.to);
      return d ? line(a, mid(t, d), 2) : null;
    }
    case "convoy": {
      const a = pos(arrow.from);
      const t = pos(arrow.target);
      const d = pos(arrow.to);
      return a && t && d ? line(a, mid(t, d), 2) : null;
    }
    default: {
      const p = pos(arrow.at);
      if (!p) return null;
      const r = 10 * asset.scale;
      return arrow.kind === "disband" ? (
        <g className={cls}>
          <line x1={p[0] - r} y1={p[1] - r} x2={p[0] + r} y2={p[1] + r} />
          <line x1={p[0] - r} y1={p[1] + r} x2={p[0] + r} y2={p[1] - r} />
        </g>
      ) : (
        <circle className={cls} cx={p[0]} cy={p[1]} r={r} />
      );
    }
  }
}
