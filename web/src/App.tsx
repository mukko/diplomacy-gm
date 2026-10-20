import { useEffect, useState } from "react";
import { useAssets } from "./assets";
import { GameScreen } from "./components/GameScreen";
import { StartScreen } from "./components/StartScreen";
import { useI18n } from "./i18n";

/** The open game is kept in the URL hash (`#g_…`) so a reload stays on it. */
function currentGame(): string | undefined {
  const h = decodeURIComponent(location.hash.slice(1));
  return h ? h : undefined;
}

export function App() {
  const { t, lang, setLang } = useI18n();
  const { assets, error } = useAssets();
  const [gameId, setGameId] = useState(currentGame);

  useEffect(() => {
    const onHash = () => setGameId(currentGame());
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, []);

  const open = (id: string | undefined) => {
    location.hash = id ? encodeURIComponent(id) : "";
    setGameId(id);
  };

  return (
    <div className="app">
      <header>
        <h1>{t("appTitle")}</h1>
        <button className="small" onClick={() => setLang(lang === "ja" ? "en" : "ja")}>
          {t("language")}
        </button>
      </header>
      <main>
        {error && <p className="error">{error}</p>}
        {!gameId && <StartScreen onOpen={open} />}
        {gameId && assets && <GameScreen key={gameId} id={gameId} assets={assets} onBack={() => open(undefined)} />}
      </main>
    </div>
  );
}
