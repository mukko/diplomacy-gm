// Typed access to the standard map (docs/en/map.md). Pure data helpers, no game logic.
import data from "./standard.json";

export type PowerId = "AUS" | "ENG" | "FRA" | "GER" | "ITA" | "RUS" | "TUR";
export type ProvinceType = "land" | "coast" | "sea";
export type CoastId = "NC" | "SC" | "EC";
export type UnitType = "A" | "F";

export interface Province {
  id: string;
  name: string;
  type: ProvinceType;
  supplyCenter: boolean;
  home: PowerId | null;
  /** Present only for split-coast provinces (R-MAP-06). */
  coasts?: CoastId[];
}

export interface MapData {
  id: string;
  name: string;
  powers: PowerId[];
  provinces: Province[];
  adjacency: {
    /** Province ID → adjacent province IDs (armies never have a coast). */
    army: Record<string, string[]>;
    /**
     * Location → adjacent locations. A location is a province ID, or `Prov/Coast`
     * for split-coast provinces (which never appear without their coast here).
     */
    fleet: Record<string, string[]>;
  };
}

export const standardMap = data as MapData;

const provincesById = new Map(standardMap.provinces.map((p) => [p.id, p]));

export function getProvince(id: string): Province | undefined {
  return provincesById.get(id);
}

/** `"Bul/EC"` → `{ province: "Bul", coast: "EC" }`; `"Con"` → `{ province: "Con" }`. */
export function parseLocation(location: string): { province: string; coast?: CoastId } {
  const [province, coast] = location.split("/") as [string, CoastId | undefined];
  return coast ? { province, coast } : { province };
}

/** Locations a unit can move to directly (no convoy), R-MOV-01/02. */
export function adjacentLocations(unit: UnitType, from: string): string[] {
  const table = unit === "A" ? standardMap.adjacency.army : standardMap.adjacency.fleet;
  return table[from] ?? [];
}

export function canMove(unit: UnitType, from: string, to: string): boolean {
  return adjacentLocations(unit, from).includes(to);
}

/** Coasts of `toProvince` a fleet at `from` can reach directly (empty if not a split-coast target). */
export function reachableCoasts(from: string, toProvince: string): CoastId[] {
  return adjacentLocations("F", from)
    .map(parseLocation)
    .filter((l) => l.province === toProvince && l.coast !== undefined)
    .map((l) => l.coast as CoastId);
}

export type FleetDestination =
  | { ok: true; location: string }
  | { ok: false; reason: "not-adjacent" | "coast-required" | "coast-unreachable" };

/**
 * Resolve a fleet move target to a concrete location (R-MAP-06/07, PR-04):
 * coast omitted and both coasts reachable → invalid; only one reachable → inferred;
 * unreachable coast given → invalid.
 */
export function resolveFleetDestination(from: string, to: string): FleetDestination {
  const { province, coast } = parseLocation(to);
  const target = getProvince(province);
  if (!target) return { ok: false, reason: "not-adjacent" };

  if (!target.coasts) {
    return canMove("F", from, province) ? { ok: true, location: province } : { ok: false, reason: "not-adjacent" };
  }

  const coasts = reachableCoasts(from, province);
  if (coasts.length === 0) return { ok: false, reason: "not-adjacent" };
  if (coast) {
    return coasts.includes(coast) ? { ok: true, location: `${province}/${coast}` } : { ok: false, reason: "coast-unreachable" };
  }
  if (coasts.length > 1) return { ok: false, reason: "coast-required" };
  return { ok: true, location: `${province}/${coasts[0]}` };
}
