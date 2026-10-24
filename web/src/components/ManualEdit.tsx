// Emergency manual correction of the current position (architecture.md § 7): units and
// supply-center owners are edited on a draft, then confirmed (POST …/manual-edit) or cancelled.
import { useI18n } from "../i18n";
import type { GameState, MapData, PowerId, Unit, UnitType } from "../types";
import { POWERS } from "../types";

export interface EditDraft {
  units: Unit[];
  supplyCenters: Record<string, PowerId | null>;
  /** Selected province. */
  selected?: string;
  /** Moving the unit of this province: the next province clicked is its destination. */
  moving?: string;
}

type Province = MapData["provinces"][number];

export function startDraft(state: GameState): EditDraft {
  return { units: state.units.map((u) => ({ ...u })), supplyCenters: { ...state.supplyCenters } };
}

/** The draft as a position the board can draw. */
export function draftState(state: GameState, d: EditDraft): GameState {
  return { ...state, units: d.units, supplyCenters: d.supplyCenters };
}

/** Unit types allowed in a province (R-MAP-02). */
export function allowedTypes(p: Province): UnitType[] {
  return p.type === "land" ? ["A"] : p.type === "sea" ? ["F"] : ["A", "F"];
}

/** A unit of `type` in province `p`, with a coast when a fleet needs one (R-MAP-06). */
function placed(power: PowerId, type: UnitType, p: Province, coast?: string): Unit {
  const c = type === "F" && p.coasts ? (coast && p.coasts.includes(coast) ? coast : p.coasts[0]) : undefined;
  return { power, type, location: p.id, ...(c ? { coast: c } : {}) };
}

/** Click on the board while editing: select a province, or finish a move. */
export function clickDraft(d: EditDraft, prov: string, map: MapData): EditDraft {
  const target = map.provinces.find((p) => p.id === prov);
  if (!target) return d;
  if (d.moving && d.moving !== prov) {
    const unit = d.units.find((u) => u.location === d.moving);
    const occupied = d.units.some((u) => u.location === prov);
    if (unit && !occupied && allowedTypes(target).includes(unit.type)) {
      const moved = placed(unit.power, unit.type, target, unit.coast);
      return { ...d, units: d.units.map((u) => (u === unit ? moved : u)), selected: prov, moving: undefined };
    }
    return { ...d, selected: prov, moving: undefined };
  }
  return { ...d, selected: prov, moving: undefined };
}

/** Human-readable list of what the draft changes. */
export function draftChanges(state: GameState, d: EditDraft): string[] {
  const label = (u: Unit) => `${u.power} ${u.type} ${u.location}${u.coast ? `/${u.coast}` : ""}`;
  const before = new Set(state.units.map(label));
  const after = new Set(d.units.map(label));
  const out: string[] = [];
  for (const u of state.units) if (!after.has(label(u))) out.push(`− ${label(u)}`);
  for (const u of d.units) if (!before.has(label(u))) out.push(`+ ${label(u)}`);
  for (const [sc, owner] of Object.entries(d.supplyCenters)) {
    if (state.supplyCenters[sc] !== owner) out.push(`${sc}: ${state.supplyCenters[sc] ?? "—"} → ${owner ?? "—"}`);
  }
  return out;
}

interface Props {
  state: GameState;
  draft: EditDraft;
  map: MapData;
  onChange: (d: EditDraft) => void;
}

export function ManualEditPanel({ state, draft, map, onChange }: Props) {
  const { t, power } = useI18n();
  const prov = map.provinces.find((p) => p.id === draft.selected);
  const unit = draft.units.find((u) => u.location === draft.selected);
  const changes = draftChanges(state, draft);

  const setUnit = (u: Unit | undefined) =>
    onChange({ ...draft, units: [...draft.units.filter((x) => x.location !== draft.selected), ...(u ? [u] : [])] });

  return (
    <div className="manual-edit">
      <h3>{t("manualEdit")}</h3>
      <p className="muted hint">{t("manualEditHint")}</p>
      {!prov && <p className="muted">{t("manualEditSelect")}</p>}
      {prov && (
        <section className="card edit-card">
          <h4>
            <code>{prov.id}</code> {prov.name} <span className="muted">({t(`provinceType_${prov.type}` as never)})</span>
          </h4>
          {draft.moving ? (
            <p>
              {t("manualEditMoving")}{" "}
              <button className="small" onClick={() => onChange({ ...draft, moving: undefined })}>
                {t("cancel")}
              </button>
            </p>
          ) : unit ? (
            <div className="edit-row">
              <select value={unit.power} onChange={(e) => setUnit({ ...unit, power: e.target.value as PowerId })}>
                {POWERS.map((p) => (
                  <option key={p} value={p}>
                    {power(p)}
                  </option>
                ))}
              </select>
              <select value={unit.type} onChange={(e) => setUnit(placed(unit.power, e.target.value as UnitType, prov, unit.coast))}>
                {allowedTypes(prov).map((ty) => (
                  <option key={ty} value={ty}>
                    {t(ty === "A" ? "army" : "fleet")}
                  </option>
                ))}
              </select>
              {unit.type === "F" && prov.coasts && (
                <select value={unit.coast} onChange={(e) => setUnit({ ...unit, coast: e.target.value })}>
                  {prov.coasts.map((c) => (
                    <option key={c} value={c}>
                      {c}
                    </option>
                  ))}
                </select>
              )}
              <button className="small" onClick={() => onChange({ ...draft, moving: prov.id })}>
                {t("manualEditMove")}
              </button>
              <button className="small danger" onClick={() => setUnit(undefined)}>
                {t("manualEditRemove")}
              </button>
            </div>
          ) : (
            <div className="edit-row">
              <span className="muted">{t("manualEditNoUnit")}</span>
              {allowedTypes(prov).map((ty) => (
                <AddUnit key={ty} type={ty} onAdd={(p) => setUnit(placed(p, ty, prov))} />
              ))}
            </div>
          )}
          {prov.supplyCenter && (
            <div className="edit-row">
              <span>{t("scOwner")}</span>
              <select
                value={draft.supplyCenters[prov.id] ?? ""}
                onChange={(e) => onChange({ ...draft, supplyCenters: { ...draft.supplyCenters, [prov.id]: (e.target.value || null) as PowerId | null } })}
              >
                <option value="">{t("none")}</option>
                {POWERS.map((p) => (
                  <option key={p} value={p}>
                    {power(p)}
                  </option>
                ))}
              </select>
            </div>
          )}
        </section>
      )}
      <h4>
        {t("manualEditChanges")} ({changes.length})
      </h4>
      {changes.length === 0 ? (
        <p className="muted">{t("manualEditNoChanges")}</p>
      ) : (
        <ul className="plain changes">
          {changes.map((c) => (
            <li key={c}>
              <code>{c}</code>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function AddUnit({ type, onAdd }: { type: UnitType; onAdd: (p: PowerId) => void }) {
  const { t, power } = useI18n();
  return (
    <select value="" onChange={(e) => e.target.value && onAdd(e.target.value as PowerId)}>
      <option value="">{t(type === "A" ? "addArmy" : "addFleet")}</option>
      {POWERS.map((p) => (
        <option key={p} value={p}>
          {power(p)}
        </option>
      ))}
    </select>
  );
}
