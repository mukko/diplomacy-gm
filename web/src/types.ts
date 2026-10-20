// JSON shapes of the API server (server/src/model.rs, server/src/api.rs, docs/en/architecture.md § 4–6).

export type PowerId = "AUS" | "ENG" | "FRA" | "GER" | "ITA" | "RUS" | "TUR";
export const POWERS: PowerId[] = ["AUS", "ENG", "FRA", "GER", "ITA", "RUS", "TUR"];

export type UnitType = "A" | "F";

export interface Unit {
  power: PowerId;
  type: UnitType;
  location: string;
  coast?: string;
}

export interface DislodgedUnit extends Unit {
  attackerFrom: string;
  retreatOptions: string[];
}

export interface GameState {
  phase: string;
  units: Unit[];
  supplyCenters: Record<string, PowerId | null>;
  dislodged: DislodgedUnit[];
  standoffProvinces: string[];
  civilDisorder: { power: PowerId; since: string }[];
  winner: PowerId | null;
}

export interface GameView {
  gameId: string;
  title: string | null;
  variant: string;
  createdAt: string;
  updatedAt: string;
  state: GameState;
  submitted: PowerId[];
  resolvedPhases: number;
}

export interface GameSummary {
  gameId: string;
  title: string | null;
  phase: string;
  updatedAt: string;
}

export type Order =
  | { power: PowerId; unit: string; type: "hold" }
  | { power: PowerId; unit: string; type: "move"; to: string; viaConvoy?: boolean }
  | { power: PowerId; unit: string; type: "support"; target: string; to?: string }
  | { power: PowerId; unit: string; type: "convoy"; target: string; to: string }
  | { power: PowerId; unit: string; type: "retreat"; to: string }
  | { power: PowerId; unit: string; type: "disband" }
  | { power: PowerId; unit: string; type: "build"; unitType: UnitType; coast?: string }
  | { power: PowerId; unit: string; type: "waive" };

export type OrderStatus = "valid" | "hold" | "disband" | "ignored";

export interface OrderCheck {
  order: Order;
  status: OrderStatus;
  reason?: string;
  rules: string[];
}

/** One item of order text read by `POST …/parse-orders`. */
export interface ParsedItem {
  text: string;
  order?: Order;
  error?: string;
  check?: OrderCheck;
}

export interface UnitOptions {
  unit: Unit;
  civilDisorder: boolean;
  orders: Order[];
}

export interface PowerOptions {
  power: PowerId;
  civilDisorder: boolean;
  adjustment?: number;
  units: UnitOptions[];
  builds: Order[];
}

export interface DisbandProposal {
  power: PowerId;
  civilDisorder: boolean;
  required: number;
  ordered: string[];
  proposed: { unit: string; distance: number }[];
}

export interface OrderResult {
  order: Order;
  valid: boolean;
  outcome: "success" | "fail" | "void" | "cut";
  reason?: string;
  rules: string[];
}

export interface TraceStep {
  step: string;
  rules: string[];
  [key: string]: unknown;
}

export interface Resolution {
  phase: string;
  orders: OrderResult[];
  trace: TraceStep[];
  nextState: GameState;
}

export interface ReplayFrame {
  index: number;
  phase: string;
  state: GameState;
  orders?: Partial<Record<PowerId, Order[]>>;
  resolution?: Resolution;
}

export type Point = [number, number];

export interface MapLayout {
  viewBox: [number, number, number, number];
  provinces: Record<string, { unit: Point; label: Point; sc: Point | null }>;
  coasts: Record<string, { unit: Point }>;
}

export interface MapData {
  provinces: { id: string; name: string; type: "land" | "coast" | "sea"; supplyCenter: boolean; home: PowerId | null; coasts?: string[] }[];
}
