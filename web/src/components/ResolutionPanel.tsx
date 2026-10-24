// Step-by-step replay of a resolution trace (adjudication.md § 4).
import { useEffect, useMemo } from "react";
import { hasMessage, useI18n } from "../i18n";
import { orderText, parseTraceOrder, type Arrow } from "../orders";
import type { GameState, Resolution, TraceStep } from "../types";
import { allArrows, boardAfter, buildUnitSteps, unitStepArrows, type StatusArrow, type UnitStep, type ViewMode } from "../unitSteps";

const isMovement = (r: Resolution) => r.phase.endsWith("M");

/**
 * What the board shows for a resolution: unit by unit (movement phases; other phases step through
 * the trace), the position before the phase, or the position after it.
 */
export function resolutionView(
  resolution: Resolution,
  before: GameState,
  mode: ViewMode,
  index: number,
): { board: GameState; arrows: StatusArrow[]; selected?: string } {
  if (mode === "before") return { board: before, arrows: allArrows(resolution) };
  if (mode === "after") return { board: resolution.nextState, arrows: allArrows(resolution) };
  if (isMovement(resolution)) {
    const steps = buildUnitSteps(before, resolution);
    const step = steps[index];
    return { board: boardAfter(before, resolution, steps, index + 1), arrows: unitStepArrows(step, resolution), selected: step?.unit.location };
  }
  const traced = stepArrows(resolution.trace[index]);
  const last = index === resolution.trace.length - 1;
  return { board: last ? resolution.nextState : before, arrows: traced.arrows, selected: traced.provinces[0] };
}

/** Arrows and emphasised provinces for one trace step, from any order notation it mentions. */
export function stepArrows(step: TraceStep | undefined): { arrows: (Arrow & { status?: "ok" | "bad" | "muted" })[]; provinces: string[] } {
  if (!step) return { arrows: [], provinces: [] };
  const arrows: (Arrow & { status?: "ok" | "bad" | "muted" })[] = [];
  const seen = new Set<string>();
  const status = step.outcome === "success" || step.outcome === "given" || step.outcome === "valid" ? "ok" : step.outcome ? "bad" : undefined;
  const visit = (v: unknown, main: boolean) => {
    if (typeof v === "string") {
      if (seen.has(v)) return;
      const a = parseTraceOrder(v);
      if (a) {
        seen.add(v);
        arrows.push({ ...a, status: main ? status : "muted" });
      }
    } else if (Array.isArray(v)) v.forEach((x) => visit(x, false));
    else if (v && typeof v === "object") Object.values(v).forEach((x) => visit(x, false));
  };
  visit(step.order, true);
  for (const [k, v] of Object.entries(step)) {
    if (!["order", "step", "rules", "positions", "province", "nextPhase"].includes(k)) visit(v, false);
  }
  const provinces = typeof step.province === "string" ? [step.province] : [];
  return { arrows, provinces };
}

interface Props {
  resolution: Resolution;
  before: GameState;
  index: number;
  onIndex: (i: number) => void;
  mode: ViewMode;
  onMode: (m: ViewMode) => void;
}

export function ResolutionPanel({ resolution, before, index, onIndex, mode, onMode }: Props) {
  const { t, phase } = useI18n();
  const movement = isMovement(resolution);
  const unitSteps = useMemo(() => (movement ? buildUnitSteps(before, resolution) : []), [movement, before, resolution]);
  const count = movement ? unitSteps.length : resolution.trace.length;

  // ← / → step through while the view is unit by unit (not while typing).
  useEffect(() => {
    if (mode !== "units") return;
    const onKey = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement;
      if (el.closest("input, textarea, select")) return;
      if (e.key === "ArrowRight" && index < count - 1) onIndex(index + 1);
      if (e.key === "ArrowLeft" && index > 0) onIndex(index - 1);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [mode, index, count, onIndex]);

  return (
    <div className="resolution">
      <h3>{phase(resolution.phase)}</h3>
      <div className="segmented view-mode">
        {(["units", "before", "after"] as ViewMode[]).map((m) => (
          <button key={m} className={mode === m ? "on" : ""} onClick={() => onMode(m)}>
            {t(m === "units" ? (movement ? "modeUnits" : "modeSteps") : m === "before" ? "modeBefore" : "modeAfter")}
          </button>
        ))}
      </div>
      {mode !== "units" && <p className="muted">{t(mode === "before" ? "beforeHint" : "afterHint")}</p>}
      {mode === "units" && movement && <UnitStepper steps={unitSteps} before={before} index={index} onIndex={onIndex} />}
      {mode === "units" && !movement && <TraceStepper resolution={resolution} index={index} onIndex={onIndex} />}
      <details open={mode !== "units"}>
        <summary>{t("orderResults")}</summary>
        <table className="results">
          <tbody>
            {resolution.orders.map((r, i) => (
              <tr key={i}>
                <td>
                  <code>{orderText(before, r.order)}</code>
                </td>
                <td className={`outcome ${r.outcome}`}>{t(`outcome_${r.outcome}`)}</td>
                <td className="muted">{r.reason}</td>
                <td>
                  <Rules rules={r.rules} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </details>
    </div>
  );
}

function Stepper({ index, count, onIndex, label }: { index: number; count: number; onIndex: (i: number) => void; label: string }) {
  const { t } = useI18n();
  const last = count - 1;
  return (
    <div className="stepper">
      <button onClick={() => onIndex(0)} disabled={index <= 0}>
        {t("first")}
      </button>
      <button onClick={() => onIndex(index - 1)} disabled={index <= 0}>
        ◀ {t("prev")}
      </button>
      <span>
        {label} {Math.min(index + 1, count)} / {count}
      </span>
      <button onClick={() => onIndex(index + 1)} disabled={index >= last}>
        {t("next")} ▶
      </button>
      <button onClick={() => onIndex(last)} disabled={index >= last}>
        {t("last")}
      </button>
    </div>
  );
}

function UnitStepper({ steps, before, index, onIndex }: { steps: UnitStep[]; before: GameState; index: number; onIndex: (i: number) => void }) {
  const { t, power } = useI18n();
  const step = steps[index];
  const orderPart = (s: UnitStep) => (s.result ? orderText(before, s.result.order).slice(s.label.length).trim() || "H" : t("noOrder"));
  // A unit that only holds (ordered to, or with no order) "holds" rather than "succeeds".
  const outcome = (s: UnitStep) =>
    s.dislodged ? "dislodged" : !s.result || s.result.order.type === "hold" ? "held" : s.result.outcome;
  return (
    <>
      <Stepper index={index} count={steps.length} onIndex={onIndex} label={t("unitStep")} />
      {step && (
        <div className="step-details unit-step">
          <h4>
            <span className={`swatch ${step.unit.power}`} /> {power(step.unit.power)} <code>{step.label}</code> <code>{orderPart(step)}</code>
          </h4>
          <p>
            <span className={`outcome ${outcome(step)}`}>{t(`outcome_${outcome(step)}` as never)}</span>
            {step.result?.reason && <span className="muted"> — {step.result.reason}</span>} {step.result && <Rules rules={step.result.rules} />}
          </p>
          {step.dislodged && (
            <p>
              {t("dislodgedBy", { from: step.dislodged.attackerFrom })} · {t("retreatOptions")}:{" "}
              {step.dislodged.retreatOptions.length ? step.dislodged.retreatOptions.map((o) => <code key={o}>{o}</code>) : t("noneDisbanded")}
            </p>
          )}
          {step.related.length > 0 && (
            <details>
              <summary>{t("howDecided")}</summary>
              {step.related.map((s, i) => (
                <StepDetails key={i} step={s} />
              ))}
            </details>
          )}
        </div>
      )}
      <ol className="steps">
        {steps.map((s, i) => (
          <li key={s.label} className={i === index ? "current" : i < index ? "done" : ""}>
            <button className="link" onClick={() => onIndex(i)}>
              <span className={`swatch ${s.unit.power}`} /> <code>{s.label}</code> <code>{orderPart(s)}</code>{" "}
              <span className={`outcome ${outcome(s)}`}>{t(`outcome_${outcome(s)}` as never)}</span>
            </button>
          </li>
        ))}
      </ol>
    </>
  );
}

function TraceStepper({ resolution, index, onIndex }: { resolution: Resolution; index: number; onIndex: (i: number) => void }) {
  const { t } = useI18n();
  const steps = resolution.trace;
  const step = steps[index];
  return (
    <>
      <Stepper index={index} count={steps.length} onIndex={onIndex} label={t("step")} />
      {step && <StepDetails step={step} />}
      <ol className="steps">
        {steps.map((s, i) => (
          <li key={i} className={i === index ? "current" : ""}>
            <button className="link" onClick={() => onIndex(i)}>
              <span className="step-name">{hasMessage(`step_${s.step}`) ? t(`step_${s.step}` as never) : s.step}</span>{" "}
              {typeof s.order === "string" && <code>{s.order}</code>}
              {typeof s.province === "string" && <code>{s.province}</code>}
              {typeof s.outcome === "string" && <span className={`outcome ${s.outcome}`}>{s.outcome}</span>}
            </button>
          </li>
        ))}
      </ol>
    </>
  );
}

function StepDetails({ step }: { step: TraceStep }) {
  const { t } = useI18n();
  const { step: name, rules, ...details } = step;
  return (
    <div className="step-details">
      <h4>
        {hasMessage(`step_${name}`) ? t(`step_${name}` as never) : name} <Rules rules={rules} />
      </h4>
      <dl>
        {Object.entries(details).map(([k, v]) => (
          <div key={k}>
            <dt>{k}</dt>
            <dd>
              <Value v={v} />
            </dd>
          </div>
        ))}
      </dl>
    </div>
  );
}

function Value({ v }: { v: unknown }) {
  if (v === null || v === undefined) return <span className="muted">—</span>;
  if (Array.isArray(v)) {
    if (v.length === 0) return <span className="muted">—</span>;
    if (v.every((x) => typeof x === "string")) return <span>{v.map((x) => <code key={x as string}>{x as string}</code>)}</span>;
    return (
      <ul className="plain">
        {v.map((x, i) => (
          <li key={i}>
            <Value v={x} />
          </li>
        ))}
      </ul>
    );
  }
  if (typeof v === "object") {
    return (
      <span className="obj">
        {Object.entries(v as Record<string, unknown>).map(([k, x]) => (
          <span key={k}>
            <em>{k}</em>: <Value v={x} />{" "}
          </span>
        ))}
      </span>
    );
  }
  return <span>{String(v)}</span>;
}

export function Rules({ rules }: { rules: string[] }) {
  return (
    <>
      {rules.map((r) => (
        <span key={r} className="rule">
          {r}
        </span>
      ))}
    </>
  );
}
