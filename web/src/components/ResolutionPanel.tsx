// Step-by-step replay of a resolution trace (adjudication.md § 4).
import { hasMessage, useI18n } from "../i18n";
import { orderText, parseTraceOrder, type Arrow } from "../orders";
import type { GameState, Resolution, TraceStep } from "../types";

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
}

export function ResolutionPanel({ resolution, before, index, onIndex }: Props) {
  const { t, phase } = useI18n();
  const steps = resolution.trace;
  const step = steps[index];
  const last = steps.length - 1;

  return (
    <div className="resolution">
      <h3>{phase(resolution.phase)}</h3>
      <div className="stepper">
        <button onClick={() => onIndex(0)} disabled={index <= 0}>
          {t("first")}
        </button>
        <button onClick={() => onIndex(index - 1)} disabled={index <= 0}>
          ◀ {t("prev")}
        </button>
        <span>
          {t("step")} {index + 1} / {steps.length}
        </span>
        <button onClick={() => onIndex(index + 1)} disabled={index >= last}>
          {t("next")} ▶
        </button>
        <button onClick={() => onIndex(last)} disabled={index >= last}>
          {t("last")}
        </button>
      </div>
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
      <details>
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
