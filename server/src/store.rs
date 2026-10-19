//! File-based persistence: one game = one JSON file `<dataDir>/<gameId>.json`.
//! The file is the source of truth; every change is written back immediately.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::Serialize;
use thiserror::Error;

use crate::game::{Game, GameError, is_valid_game_id};
use crate::model::{Phase, Power};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("invalid game id `{0}`")]
    InvalidId(String),
    #[error("game `{0}` not found")]
    NotFound(String),
    #[error("game `{0}` already exists")]
    AlreadyExists(String),
    #[error("save file is not valid JSON: {0}")]
    Parse(#[from] serde_json::Error),
    #[error(transparent)]
    Game(#[from] GameError),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSummary {
    pub game_id: String,
    pub title: Option<String>,
    pub phase: Phase,
    pub resolved_phases: usize,
    pub winner: Option<Power>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Game> for GameSummary {
    fn from(g: &Game) -> Self {
        GameSummary {
            game_id: g.game_id.clone(),
            title: g.title.clone(),
            phase: g.state.phase,
            resolved_phases: g.history.len(),
            winner: g.state.winner,
            updated_at: g.updated_at,
        }
    }
}

/// A file in the data directory that could not be read as a game.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadableSave {
    pub file: String,
    pub error: String,
}

pub struct GameStore {
    dir: PathBuf,
    /// Serialises read-modify-write cycles so concurrent requests cannot lose updates.
    lock: Mutex<()>,
}

impl GameStore {
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(GameStore {
            dir,
            lock: Mutex::new(()),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path(&self, id: &str) -> Result<PathBuf, StoreError> {
        if !is_valid_game_id(id) {
            return Err(StoreError::InvalidId(id.to_string()));
        }
        Ok(self.dir.join(format!("{id}.json")))
    }

    /// Parse and validate save data (from a file or an upload).
    pub fn parse(json: &str) -> Result<Game, StoreError> {
        let game: Game = serde_json::from_str(json)?;
        game.validate()?;
        Ok(game)
    }

    fn read(&self, id: &str) -> Result<Game, StoreError> {
        let path = self.path(id)?;
        let json = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(StoreError::NotFound(id.to_string())),
            Err(e) => return Err(e.into()),
        };
        let game = Self::parse(&json)?;
        if game.game_id != id {
            return Err(GameError::InvalidSave(format!("file {id}.json contains game `{}`", game.game_id)).into());
        }
        Ok(game)
    }

    /// Write atomically (temp file + rename) so a crash never leaves a half-written save.
    fn write(&self, game: &Game) -> Result<(), StoreError> {
        let path = self.path(&game.game_id)?;
        let tmp = path.with_extension("json.tmp");
        let mut file = fs::File::create(&tmp)?;
        serde_json::to_writer_pretty(&mut file, game)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&tmp, &path)?;
        Ok(())
    }

    /// Restore a game from its save file.
    pub fn load(&self, id: &str) -> Result<Game, StoreError> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.read(id)
    }

    /// Save a new game; fails if a file for its ID already exists.
    pub fn create(&self, game: &Game) -> Result<(), StoreError> {
        game.validate()?;
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        if self.path(&game.game_id)?.exists() {
            return Err(StoreError::AlreadyExists(game.game_id.clone()));
        }
        self.write(game)
    }

    /// Load, change and save a game in one step. Nothing is written if `f` fails.
    pub fn update<T>(&self, id: &str, f: impl FnOnce(&mut Game) -> Result<T, GameError>) -> Result<T, StoreError> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut game = self.read(id)?;
        let out = f(&mut game)?;
        self.write(&game)?;
        Ok(out)
    }

    /// All saved games (newest first), plus files that could not be restored.
    pub fn list(&self) -> Result<(Vec<GameSummary>, Vec<UnreadableSave>), StoreError> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut games = Vec::new();
        let mut unreadable = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let file = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            match self.read(id) {
                Ok(game) => games.push(GameSummary::from(&game)),
                Err(e) => unreadable.push(UnreadableSave {
                    file,
                    error: e.to_string(),
                }),
            }
        }
        games.sort_by_key(|g| std::cmp::Reverse(g.updated_at));
        unreadable.sort_by(|a, b| a.file.cmp(&b.file));
        Ok((games, unreadable))
    }
}
