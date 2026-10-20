// UI language (ja/en). Japanese terms follow docs/glossary.md.
import { createContext, useContext, useState, type ReactNode } from "react";
import type { PowerId } from "./types";

export type Lang = "ja" | "en";

const messages = {
  ja: {
    appTitle: "ディプロマシー GM",
    language: "English",
    newGame: "新しいゲーム",
    title: "タイトル",
    create: "作成",
    savedGames: "保存済みのゲーム",
    noGames: "保存済みのゲームはありません",
    open: "開く",
    importSave: "セーブファイルを読み込む",
    importAsNew: "新しい ID で読み込む",
    unreadable: "読み込めないセーブファイル",
    back: "ゲーム一覧へ",
    exportSave: "セーブファイルを書き出す",
    mapView: "地図",
    graphView: "グラフ",
    progress: "進行状況",
    orders: "命令",
    resolution: "解決結果",
    power: "列強",
    units: "ユニット",
    supplyCenters: "補給都市",
    submitted: "提出",
    civilDisorder: "内乱",
    declare: "宣言",
    lift: "解除",
    resolvePhase: "このフェイズを解決",
    resolving: "解決中…",
    cancel: "キャンセル",
    confirmResolve: "{phase} を解決します。よろしいですか？",
    winner: "勝者",
    selectUnit: "地図か一覧からユニットを選んでください",
    choices: "命令の候補",
    noChoices: "このユニットに出せる命令はありません",
    submit: "提出",
    clear: "クリア",
    notSubmitted: "未提出",
    draftChanged: "未提出の変更あり",
    noOrder: "命令なし（維持）",
    noOrderRetreat: "命令なし（解散）",
    noOrderAdjustment: "—",
    history: "履歴",
    currentPosition: "現在の局面",
    loading: "読み込み中…",
    "step_default-disbands": "自動の解散",
    status_valid: "有効",
    status_hold: "無効 → 維持",
    status_disband: "無効 → 解散",
    status_ignored: "無視",
    adjustment: "増減",
    builds: "建設",
    disbandsRequired: "解散が必要: {n}",
    buildsAllowed: "建設できる数: {n}",
    defaultDisbands: "自動で選ばれる解散",
    distance: "距離",
    dislodgedUnits: "撃退されたユニット",
    step: "ステップ",
    prev: "前へ",
    next: "次へ",
    first: "最初",
    last: "最後",
    orderResults: "命令ごとの結果",
    closeResolution: "閉じる",
    noResolution: "まだ解決していません",
    nextPhase: "次のフェイズ",
    error: "エラー",
    type_hold: "維持",
    type_move: "移動",
    type_support: "支援",
    type_convoy: "輸送",
    type_retreat: "撤退",
    type_disband: "解散",
    type_build: "建設",
    type_waive: "建設放棄",
    viaConvoy: "輸送経由",
    outcome_success: "成功",
    outcome_fail: "失敗",
    outcome_void: "無効",
    outcome_cut: "カット",
    step_validate: "命令の検証",
    "step_convoy-routes": "輸送経路",
    "step_support-cuts": "支援カット",
    step_strengths: "戦力",
    step_moves: "移動の判定",
    step_dislodgements: "撃退",
    step_result: "結果",
    step_conflicts: "競合",
    "step_sc-ownership": "補給都市の支配",
    "step_victory-check": "勝利判定",
    season_S: "春",
    season_F: "秋",
    season_W: "冬",
    kind_M: "移動",
    kind_R: "撤退",
    kind_A: "調整",
    phaseFormat: "{year}年 {season}（{kind}）",
  },
  en: {
    appTitle: "Diplomacy GM",
    language: "日本語",
    newGame: "New game",
    title: "Title",
    create: "Create",
    savedGames: "Saved games",
    noGames: "No saved games",
    open: "Open",
    importSave: "Load a save file",
    importAsNew: "Load under a new ID",
    unreadable: "Unreadable save files",
    back: "Games",
    exportSave: "Download save file",
    mapView: "Map",
    graphView: "Graph",
    progress: "Progress",
    orders: "Orders",
    resolution: "Resolution",
    power: "Power",
    units: "Units",
    supplyCenters: "Supply centers",
    submitted: "Submitted",
    civilDisorder: "Civil disorder",
    declare: "Declare",
    lift: "Lift",
    resolvePhase: "Resolve this phase",
    resolving: "Resolving…",
    cancel: "Cancel",
    confirmResolve: "Resolve {phase}?",
    winner: "Winner",
    selectUnit: "Select a unit on the map or in the list",
    choices: "Possible orders",
    noChoices: "This unit has no orders to give",
    submit: "Submit",
    clear: "Clear",
    notSubmitted: "Not submitted",
    draftChanged: "Unsubmitted changes",
    noOrder: "No order (holds)",
    noOrderRetreat: "No order (disbanded)",
    noOrderAdjustment: "—",
    history: "History",
    currentPosition: "Current position",
    loading: "Loading…",
    "step_default-disbands": "Automatic disbands",
    status_valid: "Valid",
    status_hold: "Invalid → holds",
    status_disband: "Invalid → disbanded",
    status_ignored: "Ignored",
    adjustment: "Adjustment",
    builds: "Builds",
    disbandsRequired: "Must disband: {n}",
    buildsAllowed: "May build: {n}",
    defaultDisbands: "Automatic disbands",
    distance: "Distance",
    dislodgedUnits: "Dislodged units",
    step: "Step",
    prev: "Prev",
    next: "Next",
    first: "First",
    last: "Last",
    orderResults: "Results per order",
    closeResolution: "Close",
    noResolution: "Nothing resolved yet",
    nextPhase: "Next phase",
    error: "Error",
    type_hold: "Hold",
    type_move: "Move",
    type_support: "Support",
    type_convoy: "Convoy",
    type_retreat: "Retreat",
    type_disband: "Disband",
    type_build: "Build",
    type_waive: "Waive",
    viaConvoy: "via convoy",
    outcome_success: "succeeds",
    outcome_fail: "fails",
    outcome_void: "void",
    outcome_cut: "cut",
    step_validate: "Validation",
    "step_convoy-routes": "Convoy routes",
    "step_support-cuts": "Support cuts",
    step_strengths: "Strengths",
    step_moves: "Moves",
    step_dislodgements: "Dislodgements",
    step_result: "Result",
    step_conflicts: "Conflicts",
    "step_sc-ownership": "Supply-center ownership",
    "step_victory-check": "Victory check",
    season_S: "Spring",
    season_F: "Fall",
    season_W: "Winter",
    kind_M: "movement",
    kind_R: "retreats",
    kind_A: "adjustments",
    phaseFormat: "{season} {year} ({kind})",
  },
} as const;

export type MessageKey = keyof (typeof messages)["ja"];

const powerNames: Record<Lang, Record<PowerId, string>> = {
  ja: { AUS: "オーストリア", ENG: "イギリス", FRA: "フランス", GER: "ドイツ", ITA: "イタリア", RUS: "ロシア", TUR: "トルコ" },
  en: { AUS: "Austria", ENG: "England", FRA: "France", GER: "Germany", ITA: "Italy", RUS: "Russia", TUR: "Turkey" },
};

function format(template: string, vars: Record<string, string | number> = {}): string {
  return template.replace(/\{(\w+)\}/g, (_, k: string) => String(vars[k] ?? `{${k}}`));
}

export interface I18n {
  lang: Lang;
  setLang: (lang: Lang) => void;
  t: (key: MessageKey, vars?: Record<string, string | number>) => string;
  power: (p: PowerId) => string;
  phase: (phase: string) => string;
}

const Ctx = createContext<I18n | null>(null);

function initialLang(): Lang {
  try {
    const saved = localStorage.getItem("lang");
    if (saved === "ja" || saved === "en") return saved;
  } catch {
    // storage unavailable: fall back to the browser language
  }
  return navigator.language.startsWith("ja") ? "ja" : "en";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(initialLang);
  const setLang = (l: Lang) => {
    setLangState(l);
    document.documentElement.lang = l;
    try {
      localStorage.setItem("lang", l);
    } catch {
      // ignore
    }
  };
  const t: I18n["t"] = (key, vars) => format(messages[lang][key] ?? key, vars);
  const value: I18n = {
    lang,
    setLang,
    t,
    power: (p) => powerNames[lang][p],
    phase: (phase) => {
      const m = /^([SFW])(\d{4})([MRA])$/.exec(phase);
      if (!m) return phase;
      return t("phaseFormat", {
        year: m[2]!,
        season: t(`season_${m[1]}` as MessageKey),
        kind: t(`kind_${m[3]}` as MessageKey),
      });
    },
  };
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useI18n(): I18n {
  const v = useContext(Ctx);
  if (!v) throw new Error("useI18n outside I18nProvider");
  return v;
}

/** Translate a key that may not exist (e.g. a trace step name from the server). */
export function hasMessage(key: string): key is MessageKey {
  return key in messages.ja;
}
