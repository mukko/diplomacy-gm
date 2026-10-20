// Start screen: new game, open a saved game, load a save file (architecture.md § 7).
import { useEffect, useState, type ChangeEvent } from "react";
import { api } from "../api";
import { useI18n } from "../i18n";
import type { GameSummary } from "../types";

export function StartScreen({ onOpen }: { onOpen: (id: string) => void }) {
  const { t, phase } = useI18n();
  const [games, setGames] = useState<GameSummary[]>([]);
  const [unreadable, setUnreadable] = useState<unknown[]>([]);
  const [title, setTitle] = useState("");
  const [asNew, setAsNew] = useState(false);
  const [error, setError] = useState<string>();

  useEffect(() => {
    api.listGames().then(
      (r) => {
        setGames(r.games);
        setUnreadable(r.unreadable);
      },
      (e: unknown) => setError(String(e)),
    );
  }, []);

  const create = async () => {
    try {
      const g = await api.createGame(title.trim());
      onOpen(g.gameId);
    } catch (e) {
      setError(String(e));
    }
  };

  const importFile = async (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    try {
      const g = await api.importGame(await file.text(), asNew);
      onOpen(g.gameId);
    } catch (err) {
      setError(String(err));
    } finally {
      e.target.value = "";
    }
  };

  return (
    <div className="start">
      {error && <p className="error">{t("error")}: {error}</p>}
      <section className="card">
        <h2>{t("newGame")}</h2>
        <div className="row">
          <input value={title} placeholder={t("title")} onChange={(e) => setTitle(e.target.value)} onKeyDown={(e) => e.key === "Enter" && create()} />
          <button className="primary" onClick={create}>
            {t("create")}
          </button>
        </div>
      </section>
      <section className="card">
        <h2>{t("savedGames")}</h2>
        {games.length === 0 ? (
          <p className="muted">{t("noGames")}</p>
        ) : (
          <ul className="games">
            {games.map((g) => (
              <li key={g.gameId}>
                <button className="link" onClick={() => onOpen(g.gameId)}>
                  <strong>{g.title || g.gameId}</strong>
                  <span className="muted">
                    {phase(g.phase)} · {new Date(g.updatedAt).toLocaleString()}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
        {unreadable.length > 0 && (
          <details>
            <summary>
              {t("unreadable")} ({unreadable.length})
            </summary>
            <pre>{JSON.stringify(unreadable, null, 2)}</pre>
          </details>
        )}
      </section>
      <section className="card">
        <h2>{t("importSave")}</h2>
        <div className="row">
          <input type="file" accept="application/json,.json" onChange={importFile} />
          <label>
            <input type="checkbox" checked={asNew} onChange={(e) => setAsNew(e.target.checked)} /> {t("importAsNew")}
          </label>
        </div>
      </section>
    </div>
  );
}
