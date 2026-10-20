// Game screen: board (map / graph), progress, order entry and resolution replay.
import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "../api";
import type { Assets } from "../assets";
import { useI18n } from "../i18n";
import { orderArrow, orderText, province, sameOrder, unitLocation, type Arrow } from "../orders";
import type { DisbandProposal, GameState, GameView, Order, OrderCheck, PowerId, PowerOptions, Resolution, UnitOptions } from "../types";
import { POWERS } from "../types";
import { Board } from "./Board";
import { ResolutionPanel, Rules, stepArrows } from "./ResolutionPanel";

type Tab = "progress" | "orders" | "resolution";
type Drafts = Partial<Record<PowerId, Order[]>>;
type Checks = Partial<Record<PowerId, OrderCheck[]>>;

export function GameScreen({ id, assets, onBack }: { id: string; assets: Assets; onBack: () => void }) {
  const { t, power: powerName, phase: phaseName } = useI18n();
  const [game, setGame] = useState<GameView>();
  const [checks, setChecks] = useState<Checks>({});
  const [drafts, setDrafts] = useState<Drafts>({});
  const [active, setActive] = useState<PowerId>("AUS");
  const [selected, setSelected] = useState<string>();
  const [unitOpts, setUnitOpts] = useState<UnitOptions>();
  const [powerOpts, setPowerOpts] = useState<PowerOptions>();
  const [disbands, setDisbands] = useState<DisbandProposal[]>([]);
  const [view, setView] = useState<"map" | "graph">("map");
  const [tab, setTab] = useState<Tab>("orders");
  const [replay, setReplay] = useState<{ resolution: Resolution; before: GameState; index: number }>();
  const [busy, setBusy] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [error, setError] = useState<string>();

  const fail = (e: unknown) => setError(String(e instanceof Error ? e.message : e));

  const reload = useCallback(async () => {
    const g = await api.game(id);
    const o = await api.orders(id, g.state.phase);
    setGame(g);
    setChecks(o.orders);
    setDrafts(Object.fromEntries(Object.entries(o.orders).map(([p, cs]) => [p, cs!.map((c) => c.order)])));
    setSelected(undefined);
    setUnitOpts(undefined);
  }, [id]);

  useEffect(() => {
    reload().catch(fail);
  }, [reload]);

  // Show the most recently resolved phase when a game is opened.
  useEffect(() => {
    api
      .game(id)
      .then(async (g) => {
        if (g.resolvedPhases === 0) return;
        const { frame } = await api.replayFrame(id, g.resolvedPhases - 1);
        if (frame.resolution) setReplay((r) => r ?? { resolution: frame.resolution!, before: frame.state, index: 0 });
      })
      .catch(fail);
  }, [id]);

  const state = game?.state;
  const kind = state?.phase.slice(-1);

  // Adjustment phase: builds/disbands per power and the automatic disbands.
  useEffect(() => {
    if (!state || kind !== "A") {
      setPowerOpts(undefined);
      setDisbands([]);
      return;
    }
    api.powerOptions(id, active).then((r) => setPowerOpts(r.options), fail);
    api.defaultDisbands(id, state.phase).then((r) => setDisbands(r.proposals), fail);
  }, [id, state, kind, active, checks]);

  const selectUnit = async (loc: string) => {
    if (!state) return;
    try {
      const r = await api.unitOptions(id, province(loc));
      setSelected(province(loc));
      setUnitOpts(r.options);
      setActive(r.options.unit.power);
      setTab("orders");
    } catch {
      // no unit to order there
      setSelected(undefined);
      setUnitOpts(undefined);
    }
  };

  const setOrder = (o: Order) => {
    setDrafts((d) => {
      const list = (d[o.power] ?? []).filter((x) => !(x.unit === o.unit && x.type !== "build" && o.type !== "build") && !(o.type === "build" && x.unit === o.unit));
      return { ...d, [o.power]: [...list, o] };
    });
  };

  const removeOrder = (p: PowerId, o: Order) => setDrafts((d) => ({ ...d, [p]: (d[p] ?? []).filter((x) => !sameOrder(x, o)) }));

  // Board clicks: pick a target for the selected unit, otherwise select the unit there.
  const onBoardSelect = (loc: string) => {
    if (tab === "resolution" && replay) return;
    if (unitOpts && selected) {
      const targets = unitOpts.orders.filter((o) => (o.type === "move" || o.type === "retreat") && !("viaConvoy" in o && o.viaConvoy));
      const exact = targets.filter((o) => "to" in o && o.to === loc);
      const byProvince = targets.filter((o) => "to" in o && province(o.to!) === province(loc));
      const pick = exact[0] ?? (byProvince.length === 1 ? byProvince[0] : undefined);
      if (pick) {
        setOrder(pick);
        return;
      }
    }
    if (kind === "A" && powerOpts && state) {
      const builds = powerOpts.builds.filter((o) => o.unit === province(loc));
      if (builds.length > 0 && !state.units.some((u) => u.location === province(loc))) {
        setSelected(province(loc));
        setUnitOpts(undefined);
        return;
      }
    }
    void selectUnit(loc);
  };

  const submit = async (p: PowerId) => {
    if (!state) return;
    setBusy(true);
    try {
      const r = await api.submitOrders(id, state.phase, p, drafts[p] ?? []);
      setChecks((c) => ({ ...c, [p]: r.results }));
      setError(undefined);
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  const resolve = async () => {
    if (!state) return;
    setConfirming(false);
    setBusy(true);
    try {
      const resolution = await api.resolve(id, state.phase);
      setReplay({ resolution, before: state, index: 0 });
      setTab("resolution");
      await reload();
      setError(undefined);
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  const toggleDisorder = async (p: PowerId, on: boolean) => {
    try {
      await (on ? api.declareCivilDisorder(id, p) : api.liftCivilDisorder(id, p));
      await reload();
    } catch (e) {
      fail(e);
    }
  };

  // What the board shows.
  const boardState = tab === "resolution" && replay ? (replay.index === replay.resolution.trace.length - 1 ? replay.resolution.nextState : replay.before) : state;
  const step = tab === "resolution" && replay ? replay.resolution.trace[replay.index] : undefined;
  const traced = stepArrows(step);

  const draftArrows = useMemo(() => {
    if (!state || tab === "resolution") return [] as (Arrow & { status?: "ok" | "bad" | "muted" })[];
    const out: (Arrow & { status?: "ok" | "bad" | "muted" })[] = [];
    for (const p of POWERS) {
      for (const o of drafts[p] ?? []) {
        if (o.type === "hold" || o.type === "waive") continue;
        const check = checks[p]?.find((c) => sameOrder(c.order, o));
        const status = !check ? "muted" : check.status === "valid" ? "ok" : "bad";
        out.push({ ...orderArrow(o), status: p === active ? status : "muted" });
      }
    }
    return out;
  }, [state, drafts, checks, active, tab]);

  const targets = useMemo(() => {
    if (!unitOpts) return [];
    return unitOpts.orders.flatMap((o) => ((o.type === "move" || o.type === "retreat") && !("viaConvoy" in o && o.viaConvoy) ? [o.to] : []));
  }, [unitOpts]);

  if (error && !game) return <p className="error">{error}</p>;
  if (!game || !state || !boardState) return <p className="muted">…</p>;

  const asset = view === "map" ? assets.map : assets.graph;
  const disorder = new Set(state.civilDisorder.map((c) => c.power));

  return (
    <div className="game">
      <div className="toolbar">
        <button onClick={onBack}>← {t("back")}</button>
        <h2>
          {game.title || game.gameId} <span className="phase">{phaseName(state.phase)}</span>
        </h2>
        <div className="spacer" />
        <div className="segmented">
          <button className={view === "map" ? "on" : ""} onClick={() => setView("map")}>
            {t("mapView")}
          </button>
          <button className={view === "graph" ? "on" : ""} onClick={() => setView("graph")}>
            {t("graphView")}
          </button>
        </div>
        <a className="button" href={api.saveUrl(id)} download>
          {t("exportSave")}
        </a>
        <button className="primary" onClick={() => setConfirming(true)} disabled={busy || confirming || !!state.winner}>
          {busy ? t("resolving") : t("resolvePhase")}
        </button>
      </div>
      {confirming && (
        <div className="confirm">
          <span>{t("confirmResolve", { phase: phaseName(state.phase) })}</span>
          <button className="primary" onClick={resolve}>
            {t("resolvePhase")}
          </button>
          <button onClick={() => setConfirming(false)}>{t("cancel")}</button>
        </div>
      )}
      {error && <p className="error">{t("error")}: {error}</p>}
      {state.winner && (
        <p className="winner">
          {t("winner")}: {powerName(state.winner)}
        </p>
      )}
      <div className="layout">
        <div className="board-wrap">
          <Board
            asset={asset}
            state={boardState}
            selected={tab === "resolution" ? traced.provinces[0] : selected}
            targets={tab === "resolution" ? [] : targets}
            arrows={tab === "resolution" ? traced.arrows : draftArrows}
            onSelect={onBoardSelect}
          />
        </div>
        <aside className="side">
          <div className="tabs">
            {(["progress", "orders", "resolution"] as Tab[]).map((k) => (
              <button key={k} className={tab === k ? "on" : ""} onClick={() => setTab(k)}>
                {t(k)}
              </button>
            ))}
          </div>
          {tab === "progress" && <Progress state={state} submitted={game.submitted} disorder={disorder} onToggle={toggleDisorder} />}
          {tab === "orders" && (
            <OrdersPanel
              state={state}
              active={active}
              setActive={(p) => {
                setActive(p);
                setSelected(undefined);
                setUnitOpts(undefined);
              }}
              drafts={drafts}
              checks={checks}
              disorder={disorder}
              selected={selected}
              unitOpts={unitOpts}
              powerOpts={powerOpts}
              disbands={disbands}
              busy={busy}
              onSelectUnit={selectUnit}
              onSetOrder={setOrder}
              onRemove={removeOrder}
              onSubmit={submit}
            />
          )}
          {tab === "resolution" &&
            (replay ? (
              <ResolutionPanel resolution={replay.resolution} before={replay.before} index={replay.index} onIndex={(i) => setReplay({ ...replay, index: i })} />
            ) : (
              <p className="muted">{t("noResolution")}</p>
            ))}
        </aside>
      </div>
    </div>
  );
}

function Progress({ state, submitted, disorder, onToggle }: { state: GameState; submitted: PowerId[]; disorder: Set<PowerId>; onToggle: (p: PowerId, on: boolean) => void }) {
  const { t, power } = useI18n();
  const scs = (p: PowerId) => Object.values(state.supplyCenters).filter((o) => o === p).length;
  const units = (p: PowerId) => state.units.filter((u) => u.power === p).length;
  return (
    <table className="progress">
      <thead>
        <tr>
          <th>{t("power")}</th>
          <th>{t("units")}</th>
          <th>{t("supplyCenters")}</th>
          <th>{t("submitted")}</th>
          <th>{t("civilDisorder")}</th>
        </tr>
      </thead>
      <tbody>
        {POWERS.map((p) => (
          <tr key={p}>
            <td>
              <span className={`swatch ${p}`} /> {power(p)}
            </td>
            <td>{units(p)}</td>
            <td>{scs(p)}</td>
            <td>{submitted.includes(p) ? "✓" : ""}</td>
            <td>
              <button className="small" onClick={() => onToggle(p, !disorder.has(p))}>
                {disorder.has(p) ? `${t("civilDisorder")} · ${t("lift")}` : t("declare")}
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

interface OrdersPanelProps {
  state: GameState;
  active: PowerId;
  setActive: (p: PowerId) => void;
  drafts: Drafts;
  checks: Checks;
  disorder: Set<PowerId>;
  selected?: string;
  unitOpts?: UnitOptions;
  powerOpts?: PowerOptions;
  disbands: DisbandProposal[];
  busy: boolean;
  onSelectUnit: (loc: string) => void;
  onSetOrder: (o: Order) => void;
  onRemove: (p: PowerId, o: Order) => void;
  onSubmit: (p: PowerId) => void;
}

function OrdersPanel(props: OrdersPanelProps) {
  const { t, power } = useI18n();
  const { state, active, drafts, checks, selected, unitOpts, powerOpts } = props;
  const kind = state.phase.slice(-1);
  const draft = drafts[active] ?? [];
  const submittedList = checks[active];
  const changed = JSON.stringify(draft) !== JSON.stringify(submittedList?.map((c) => c.order) ?? []);

  const myUnits = kind === "R" ? state.dislodged.filter((u) => u.power === active) : state.units.filter((u) => u.power === active);
  const orderFor = (loc: string) => draft.find((o) => o.unit === loc && o.type !== "build" && o.type !== "waive");
  const checkFor = (o: Order) => submittedList?.find((c) => sameOrder(c.order, o));

  const options = unitOpts && unitOpts.unit.power === active ? unitOpts.orders : [];
  const buildOptions = kind === "A" && powerOpts ? powerOpts.builds.filter((o) => !selected || o.unit === selected || o.type === "waive") : [];
  const myDisbands = props.disbands.find((d) => d.power === active);

  return (
    <div className="orders">
      <div className="powers">
        {POWERS.map((p) => (
          <button key={p} className={`power-tab ${p} ${p === active ? "on" : ""}`} onClick={() => props.setActive(p)} title={power(p)}>
            {p}
            {checks[p] ? " ✓" : ""}
            {props.disorder.has(p) ? " ⚠" : ""}
          </button>
        ))}
      </div>
      <h3>
        {power(active)} {props.disorder.has(active) && <span className="badge warn">{t("civilDisorder")}</span>}
      </h3>
      {kind === "A" && powerOpts?.adjustment !== undefined && (
        <p>{powerOpts.adjustment < 0 ? t("disbandsRequired", { n: -powerOpts.adjustment }) : t("buildsAllowed", { n: powerOpts.adjustment })}</p>
      )}

      <ul className="unit-list">
        {myUnits.map((u) => {
          const o = orderFor(u.location);
          const c = o && checkFor(o);
          return (
            <li key={u.location} className={selected === u.location ? "selected" : ""}>
              <button className="link" onClick={() => props.onSelectUnit(u.location)}>
                <code>
                  {u.type} {unitLocation(u)}
                </code>
              </button>
              <span className="order">{o ? <code>{orderText(state, o).slice(`${u.type} ${unitLocation(u)}`.length).trim()}</code> : <span className="muted">{t("noOrder")}</span>}</span>
              {c && <Status check={c} />}
              {o && (
                <button className="small" onClick={() => props.onRemove(active, o)}>
                  ×
                </button>
              )}
            </li>
          );
        })}
        {draft
          .filter((o) => o.type === "build" || o.type === "waive")
          .map((o) => {
            const c = checkFor(o);
            return (
              <li key={JSON.stringify(o)}>
                <code>{orderText(state, o)}</code>
                {c && <Status check={c} />}
                <button className="small" onClick={() => props.onRemove(active, o)}>
                  ×
                </button>
              </li>
            );
          })}
      </ul>

      <div className="submit-row">
        <button className="primary" onClick={() => props.onSubmit(active)} disabled={props.busy}>
          {t("submit")}
        </button>
        <span className="muted">{!submittedList ? t("notSubmitted") : changed ? t("draftChanged") : ""}</span>
      </div>

      {(options.length > 0 || buildOptions.length > 0) && (
        <section>
          <h4>{t("choices")}</h4>
          <OptionList state={state} orders={[...options, ...buildOptions]} current={draft} onPick={props.onSetOrder} />
        </section>
      )}
      {unitOpts && options.length === 0 && <p className="muted">{t("noChoices")}</p>}
      {!unitOpts && buildOptions.length === 0 && <p className="muted">{t("selectUnit")}</p>}

      {kind === "A" && myDisbands && myDisbands.required > 0 && (
        <section>
          <h4>{t("defaultDisbands")}</h4>
          <ul className="plain">
            {myDisbands.ordered.map((u) => (
              <li key={u}>
                <code>{u}</code> ✓
              </li>
            ))}
            {myDisbands.proposed.map((d) => (
              <li key={d.unit}>
                <code>{d.unit}</code> <span className="muted">({t("distance")} {d.distance})</span>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}

function OptionList({ state, orders, current, onPick }: { state: GameState; orders: Order[]; current: Order[]; onPick: (o: Order) => void }) {
  const { t } = useI18n();
  const groups = new Map<string, Order[]>();
  for (const o of orders) groups.set(o.type, [...(groups.get(o.type) ?? []), o]);
  return (
    <div className="options">
      {[...groups.entries()].map(([type, list]) => (
        <div key={type} className="option-group">
          <h5>{t(`type_${type}` as never)}</h5>
          <div className="option-buttons">
            {list.map((o) => {
              const on = current.some((c) => sameOrder(c, o));
              return (
                <button key={JSON.stringify(o)} className={on ? "on" : ""} onClick={() => onPick(o)}>
                  {orderText(state, o)}
                </button>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}

function Status({ check }: { check: OrderCheck }) {
  const { t } = useI18n();
  return (
    <span className={`status ${check.status}`} title={check.reason ?? ""}>
      {t(`status_${check.status}`)}
      {check.reason && <span className="reason"> — {check.reason}</span>} <Rules rules={check.rules} />
    </span>
  );
}
