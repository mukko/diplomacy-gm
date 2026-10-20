// Thin client for the API server (docs/en/architecture.md § 6).
import type {
  DisbandProposal,
  GameState,
  GameSummary,
  GameView,
  MapData,
  MapLayout,
  Order,
  OrderCheck,
  PowerId,
  PowerOptions,
  Resolution,
  UnitOptions,
} from "./types";

export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    message: string,
  ) {
    super(message);
  }
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(path, {
    method,
    headers: body === undefined ? undefined : { "content-type": "application/json" },
    body: body === undefined ? undefined : typeof body === "string" ? body : JSON.stringify(body),
  });
  const text = await res.text();
  const data = text ? JSON.parse(text) : null;
  if (!res.ok) throw new ApiError(res.status, data?.error ?? "error", data?.message ?? res.statusText);
  return data as T;
}

async function text(path: string): Promise<string> {
  const res = await fetch(path);
  if (!res.ok) throw new ApiError(res.status, "error", res.statusText);
  return res.text();
}

export const api = {
  map: () => request<MapData>("GET", "/maps/standard"),
  mapLayout: () => request<MapLayout>("GET", "/maps/standard/layout"),
  mapSvg: () => text("/maps/standard/svg"),
  graphSvg: () => text("/maps/standard/graph"),

  listGames: () => request<{ games: GameSummary[]; unreadable: unknown[] }>("GET", "/games"),
  createGame: (title: string) => request<GameView>("POST", "/games", { title: title || null }),
  importGame: (save: string, asNew: boolean) =>
    request<GameView>("POST", `/games/import${asNew ? "?asNew=true" : ""}`, save),
  game: (id: string) => request<GameView>("GET", `/games/${id}`),
  saveUrl: (id: string) => `/games/${id}/save`,
  replayFrame: (id: string, index: number) =>
    request<{ total: number; frame: { index: number; phase: string; state: GameState; resolution?: Resolution } }>(
      "GET",
      `/games/${id}/replay/${index}`,
    ),

  orders: (id: string, phase: string) =>
    request<{ phase: string; orders: Partial<Record<PowerId, OrderCheck[]>> }>("GET", `/games/${id}/phases/${phase}/orders`),
  submitOrders: (id: string, phase: string, power: PowerId, orders: Order[]) =>
    request<{ results: OrderCheck[] }>("PUT", `/games/${id}/phases/${phase}/orders/${power}`, orders),
  resolve: (id: string, phase: string) => request<Resolution>("POST", `/games/${id}/phases/${phase}/resolve`),

  unitOptions: (id: string, prov: string) =>
    request<{ phase: string; options: UnitOptions }>("GET", `/games/${id}/units/${prov}/options`),
  powerOptions: (id: string, power: PowerId) =>
    request<{ phase: string; options: PowerOptions }>("GET", `/games/${id}/powers/${power}/options`),
  defaultDisbands: (id: string, phase: string) =>
    request<{ phase: string; proposals: DisbandProposal[] }>("GET", `/games/${id}/phases/${phase}/default-disbands`),

  declareCivilDisorder: (id: string, power: PowerId) =>
    request<{ civilDisorder: GameState["civilDisorder"] }>("PUT", `/games/${id}/civil-disorder/${power}`),
  liftCivilDisorder: (id: string, power: PowerId) =>
    request<{ civilDisorder: GameState["civilDisorder"] }>("DELETE", `/games/${id}/civil-disorder/${power}`),
};
