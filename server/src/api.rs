//! HTTP API (docs/en/architecture.md § 6).
//!
//! Start screen choices map to:
//! - new game → `POST /games`
//! - load     → `GET /games` (saved games) then `GET /games/{id}`, or `POST /games/import` with a save file
//! - replay   → `GET /games/{id}/replay` and `GET /games/{id}/replay/{index}`

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::adjudicator::{AdjudicationError, Adjudicator};
use crate::engine::options;
use crate::game::{Game, GameError, new_game_id};
use crate::map;
use crate::model::{GameState, Order, Phase, PhaseKind, Power};
use crate::store::{GameStore, StoreError};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<GameStore>,
    pub adjudicator: Arc<dyn Adjudicator>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/maps/standard", get(get_map))
        .route("/games", get(list_games).post(create_game))
        .route("/games/import", post(import_game))
        .route("/games/{id}", get(get_game))
        .route("/games/{id}/save", get(export_game))
        .route("/games/{id}/history", get(get_history))
        .route("/games/{id}/replay", get(get_replay))
        .route("/games/{id}/replay/{index}", get(get_replay_frame))
        .route("/games/{id}/phases/{phase}/orders/{power}", put(submit_orders))
        .route("/games/{id}/phases/{phase}/resolve", post(resolve_phase))
        .route("/games/{id}/phases/{phase}/default-disbands", get(get_default_disbands))
        .route("/games/{id}/units/{prov}/options", get(get_unit_options))
        .route("/games/{id}/powers/{power}/options", get(get_power_options))
        .route(
            "/games/{id}/civil-disorder/{power}",
            put(declare_civil_disorder).delete(lift_civil_disorder),
        )
        .with_state(state)
}

pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        ApiError {
            status,
            code,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({ "error": self.code, "message": self.message })),
        )
            .into_response()
    }
}

impl From<GameError> for ApiError {
    fn from(e: GameError) -> Self {
        let (status, code) = match &e {
            GameError::PhaseMismatch { .. } => (StatusCode::CONFLICT, "phase-mismatch"),
            GameError::GameOver => (StatusCode::CONFLICT, "game-over"),
            GameError::InvalidOrder(_) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid-order"),
            GameError::InvalidSave(_) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid-save"),
            GameError::Adjudication(AdjudicationError::Failed(_)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "adjudication-failed")
            }
        };
        ApiError::new(status, code, e.to_string())
    }
}

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::Game(g) => g.into(),
            StoreError::InvalidId(_) => ApiError::new(StatusCode::BAD_REQUEST, "invalid-id", e.to_string()),
            StoreError::NotFound(_) => ApiError::new(StatusCode::NOT_FOUND, "not-found", e.to_string()),
            StoreError::AlreadyExists(_) => ApiError::new(StatusCode::CONFLICT, "already-exists", e.to_string()),
            StoreError::Parse(_) => ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "invalid-save", e.to_string()),
            StoreError::Io(_) => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "io", e.to_string()),
        }
    }
}

type ApiResult<T> = Result<T, ApiError>;

fn parse_path<T: std::str::FromStr<Err = String>>(s: &str) -> ApiResult<T> {
    s.parse()
        .map_err(|e: String| ApiError::new(StatusCode::BAD_REQUEST, "invalid-path", e))
}

/// Public view of a game: current position and who has submitted, but not the secret orders.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    game_id: String,
    title: Option<String>,
    variant: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    state: GameState,
    submitted: Vec<Power>,
    resolved_phases: usize,
}

impl From<Game> for GameView {
    fn from(g: Game) -> Self {
        GameView {
            submitted: g.orders.keys().copied().collect(),
            resolved_phases: g.history.len(),
            game_id: g.game_id,
            title: g.title,
            variant: g.variant,
            created_at: g.created_at,
            updated_at: g.updated_at,
            state: g.state,
        }
    }
}

async fn get_map() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/json")], map::STANDARD_MAP_JSON)
}

async fn list_games(State(app): State<AppState>) -> ApiResult<impl IntoResponse> {
    let (games, unreadable) = app.store.list()?;
    Ok(Json(json!({ "games": games, "unreadable": unreadable })))
}

#[derive(Deserialize, Default)]
struct CreateGame {
    title: Option<String>,
}

async fn create_game(State(app): State<AppState>, body: Option<Json<CreateGame>>) -> ApiResult<impl IntoResponse> {
    let Json(req) = body.unwrap_or_default();
    let game = Game::new_standard(req.title);
    app.store.create(&game)?;
    Ok((StatusCode::CREATED, Json(GameView::from(game))))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportQuery {
    #[serde(default)]
    as_new: bool,
}

/// Restore a game from an uploaded save file. `?asNew=true` stores it under a fresh ID.
async fn import_game(
    State(app): State<AppState>,
    Query(q): Query<ImportQuery>,
    body: String,
) -> ApiResult<impl IntoResponse> {
    let mut game = GameStore::parse(&body)?;
    if q.as_new {
        game.game_id = new_game_id();
        game.updated_at = Utc::now();
    }
    app.store.create(&game)?;
    Ok((StatusCode::CREATED, Json(GameView::from(game))))
}

async fn get_game(State(app): State<AppState>, Path(id): Path<String>) -> ApiResult<impl IntoResponse> {
    Ok(Json(GameView::from(app.store.load(&id)?)))
}

/// Download the complete save file, including unresolved orders (local single-GM tool, no access control).
async fn export_game(State(app): State<AppState>, Path(id): Path<String>) -> ApiResult<impl IntoResponse> {
    let game = app.store.load(&id)?;
    let body = serde_json::to_string_pretty(&game).map_err(StoreError::from)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{id}.json\""),
            ),
        ],
        body,
    ))
}

async fn get_history(State(app): State<AppState>, Path(id): Path<String>) -> ApiResult<impl IntoResponse> {
    let game = app.store.load(&id)?;
    Ok(Json(json!({ "gameId": game.game_id, "history": game.history })))
}

async fn get_replay(State(app): State<AppState>, Path(id): Path<String>) -> ApiResult<impl IntoResponse> {
    let game = app.store.load(&id)?;
    Ok(Json(json!({ "gameId": game.game_id, "frames": game.replay_frames() })))
}

async fn get_replay_frame(
    State(app): State<AppState>,
    Path((id, index)): Path<(String, usize)>,
) -> ApiResult<impl IntoResponse> {
    let game = app.store.load(&id)?;
    let frame = game.replay_frame(index).ok_or_else(|| {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("replay frame {index} does not exist ({} frames)", game.replay_len()),
        )
    })?;
    Ok(Json(
        json!({ "gameId": game.game_id, "total": game.replay_len(), "frame": frame }),
    ))
}

async fn submit_orders(
    State(app): State<AppState>,
    Path((id, phase, power)): Path<(String, String, String)>,
    Json(orders): Json<Vec<Order>>,
) -> ApiResult<impl IntoResponse> {
    let phase: Phase = parse_path(&phase)?;
    let power: Power = parse_path(&power)?;
    let count = orders.len();
    app.store.update(&id, |g| g.submit_orders(phase, power, orders))?;
    Ok(Json(json!({ "phase": phase, "power": power, "accepted": count })))
}

async fn resolve_phase(
    State(app): State<AppState>,
    Path((id, phase)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let phase: Phase = parse_path(&phase)?;
    let adjudicator = app.adjudicator.clone();
    let resolution = app.store.update(&id, |g| {
        g.resolve(phase, adjudicator.as_ref()).map(|r| r.resolution.clone())
    })?;
    Ok(Json(resolution))
}

/// Legal orders for one unit in the current phase (a dislodged unit in a retreat phase).
async fn get_unit_options(
    State(app): State<AppState>,
    Path((id, prov)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let game = app.store.load(&id)?;
    let opts = options::unit_options(&game.state, &prov).ok_or_else(|| {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no unit to order in {prov} in {}", game.state.phase),
        )
    })?;
    Ok(Json(json!({ "phase": game.state.phase, "options": opts })))
}

/// Legal orders for all of a power's units, plus its builds in an adjustment phase.
async fn get_power_options(
    State(app): State<AppState>,
    Path((id, power)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let power: Power = parse_path(&power)?;
    let game = app.store.load(&id)?;
    Ok(Json(
        json!({ "phase": game.state.phase, "options": options::power_options(&game.state, power) }),
    ))
}

/// Disbands the engine would choose (R-CD-03 / PR-08), after the disbands already submitted.
async fn get_default_disbands(
    State(app): State<AppState>,
    Path((id, phase)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let phase: Phase = parse_path(&phase)?;
    let game = app.store.load(&id)?;
    if phase != game.state.phase {
        return Err(GameError::PhaseMismatch {
            expected: game.state.phase,
            got: phase,
        }
        .into());
    }
    if phase.kind != PhaseKind::Adjustment {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "not-adjustment-phase",
            format!("{phase} is not an adjustment phase"),
        ));
    }
    let orders: Vec<Order> = game.orders.values().flatten().cloned().collect();
    Ok(Json(
        json!({ "phase": phase, "proposals": options::default_disbands(&game.state, &orders) }),
    ))
}

/// Declare civil disorder for a power from the current phase (PR-16).
async fn declare_civil_disorder(
    State(app): State<AppState>,
    Path((id, power)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let power: Power = parse_path(&power)?;
    let (changed, state) = app.store.update(&id, |g| {
        let changed = g.declare_civil_disorder(power)?;
        Ok((changed, g.state.clone()))
    })?;
    Ok(Json(
        json!({ "power": power, "changed": changed, "phase": state.phase, "civilDisorder": state.civil_disorder }),
    ))
}

/// Lift civil disorder for a power from the current phase on (PR-16).
async fn lift_civil_disorder(
    State(app): State<AppState>,
    Path((id, power)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let power: Power = parse_path(&power)?;
    let (changed, state) = app.store.update(&id, |g| {
        let changed = g.lift_civil_disorder(power)?;
        Ok((changed, g.state.clone()))
    })?;
    Ok(Json(
        json!({ "power": power, "changed": changed, "phase": state.phase, "civilDisorder": state.civil_disorder }),
    ))
}
