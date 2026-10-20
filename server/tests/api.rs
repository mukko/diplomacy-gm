//! API tests: new game, save (one JSON file per game), restore, replay, import/export.

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use diplomacy_gm_server::adjudicator::{AdjudicationError, Adjudicator};
use diplomacy_gm_server::api::{AppState, router};
use diplomacy_gm_server::engine::Engine;
use diplomacy_gm_server::game::Game;
use diplomacy_gm_server::model::{GameState, Order, OrderResult, Outcome, Phase, Resolution, TraceStep};
use diplomacy_gm_server::store::GameStore;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

/// Stand-in for the engine: every order succeeds and nothing moves; the game advances
/// S → F → W → S. Enough to exercise history, saving and replay.
struct FakeEngine;

impl Adjudicator for FakeEngine {
    fn adjudicate(&self, state: &GameState, orders: &[Order]) -> Result<Resolution, AdjudicationError> {
        let p = state.phase;
        let next = match p.to_string().as_bytes()[0] {
            b'S' => format!("F{}M", p.year),
            b'F' => format!("W{}A", p.year),
            _ => format!("S{}M", p.year + 1),
        };
        let mut next_state = state.clone();
        next_state.phase = next.parse::<Phase>().unwrap();
        let mut details = serde_json::Map::new();
        details.insert("order".into(), json!(format!("{} orders", orders.len())));
        Ok(Resolution {
            phase: p,
            orders: orders
                .iter()
                .map(|o| OrderResult {
                    order: o.clone(),
                    valid: true,
                    outcome: Outcome::Success,
                    reason: None,
                    rules: vec![],
                })
                .collect(),
            trace: vec![TraceStep {
                step: "fake".into(),
                rules: vec!["R-MOV-04".into()],
                details,
            }],
            next_state,
        })
    }
}

fn app(dir: &Path, adjudicator: Arc<dyn Adjudicator>) -> Router {
    router(AppState {
        store: Arc::new(GameStore::open(dir).unwrap()),
        adjudicator,
    })
}

async fn call(app: &Router, method: Method, uri: &str, body: Option<String>) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(body.map(Body::from).unwrap_or_else(Body::empty))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

async fn new_game(app: &Router) -> String {
    let (status, body) = call(
        app,
        Method::POST,
        "/games",
        Some(json!({ "title": "Test" }).to_string()),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["gameId"].as_str().unwrap().to_string()
}

fn json_files(dir: &Path) -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".json"))
        .collect();
    files.sort();
    files
}

fn read_save(dir: &Path, id: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(dir.join(format!("{id}.json"))).unwrap()).unwrap()
}

const GERMAN_ORDERS: &str = r#"[
  { "power": "GER", "unit": "Ber", "type": "move", "to": "Kie" },
  { "power": "GER", "unit": "Mun", "type": "move", "to": "Ruh" },
  { "power": "GER", "unit": "Kie", "type": "move", "to": "Den" }
]"#;

#[tokio::test]
async fn new_game_creates_one_save_file() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path(), Arc::new(Engine));
    let id = new_game(&app).await;

    assert_eq!(json_files(dir.path()), vec![format!("{id}.json")]);
    let save = read_save(dir.path(), &id);
    assert_eq!(save["format"], "diplomacy-gm/game");
    assert_eq!(save["formatVersion"], 1);
    assert_eq!(save["state"]["phase"], "S1901M");
    assert_eq!(save["state"]["units"].as_array().unwrap().len(), 22);

    let (status, list) = call(&app, Method::GET, "/games", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["games"][0]["gameId"], id.as_str());
    assert_eq!(list["games"][0]["phase"], "S1901M");

    // A second game is a second file.
    new_game(&app).await;
    assert_eq!(json_files(dir.path()).len(), 2);
}

#[tokio::test]
async fn orders_are_saved_and_restored_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let id = {
        let app = app(dir.path(), Arc::new(Engine));
        let id = new_game(&app).await;
        let (status, body) = call(
            &app,
            Method::PUT,
            &format!("/games/{id}/phases/S1901M/orders/GER"),
            Some(GERMAN_ORDERS.into()),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        id
    };
    assert_eq!(read_save(dir.path(), &id)["orders"]["GER"].as_array().unwrap().len(), 3);

    // New server instance on the same directory = restart.
    let app = app(dir.path(), Arc::new(Engine));
    let (status, game) = call(&app, Method::GET, &format!("/games/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(game["state"]["phase"], "S1901M");
    assert_eq!(game["submitted"], json!(["GER"]));
    // The public view does not reveal the orders themselves.
    assert!(game.get("orders").is_none());
}

#[tokio::test]
async fn order_submission_is_checked() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path(), Arc::new(Engine));
    let id = new_game(&app).await;
    let before = read_save(dir.path(), &id);

    let put = |phase: &str, power: &str, body: &str| {
        let app = app.clone();
        let uri = format!("/games/{id}/phases/{phase}/orders/{power}");
        let body = body.to_string();
        async move { call(&app, Method::PUT, &uri, Some(body)).await }
    };
    assert_eq!(put("F1901M", "GER", GERMAN_ORDERS).await.0, StatusCode::CONFLICT);
    assert_eq!(put("X1901M", "GER", GERMAN_ORDERS).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(put("S1901M", "XYZ", GERMAN_ORDERS).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(
        put("S1901M", "FRA", GERMAN_ORDERS).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let no_unit = r#"[{ "power": "GER", "unit": "Par", "type": "hold" }]"#;
    assert_eq!(put("S1901M", "GER", no_unit).await.0, StatusCode::UNPROCESSABLE_ENTITY);
    let unknown = r#"[{ "power": "GER", "unit": "Swi", "type": "hold" }]"#;
    assert_eq!(put("S1901M", "GER", unknown).await.0, StatusCode::UNPROCESSABLE_ENTITY);

    assert_eq!(
        read_save(dir.path(), &id),
        before,
        "rejected orders must not change the save file"
    );
}

/// An engine that always fails, to check that a failed resolve leaves the game untouched.
struct FailingEngine;

impl Adjudicator for FailingEngine {
    fn adjudicate(&self, _state: &GameState, _orders: &[Order]) -> Result<Resolution, AdjudicationError> {
        Err(AdjudicationError::Failed("test".into()))
    }
}

#[tokio::test]
async fn failed_resolve_is_500_and_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path(), Arc::new(FailingEngine));
    let id = new_game(&app).await;
    let before = read_save(dir.path(), &id);
    let (status, body) = call(&app, Method::POST, &format!("/games/{id}/phases/S1901M/resolve"), None).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"], "adjudication-failed");
    assert_eq!(read_save(dir.path(), &id), before);
}

#[tokio::test]
async fn engine_resolves_retreat_and_adjustment_phases() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path(), Arc::new(Engine));
    let id = new_game(&app).await;
    let put = |phase: &str, power: &str, body: &str| {
        let uri = format!("/games/{id}/phases/{phase}/orders/{power}");
        let app = app.clone();
        let body = body.to_string();
        async move {
            let (status, res) = call(&app, Method::PUT, &uri, Some(body)).await;
            assert!(status.is_success(), "{res}");
        }
    };
    let resolve = |phase: &str| {
        let uri = format!("/games/{id}/phases/{phase}/resolve");
        let app = app.clone();
        async move {
            let (status, res) = call(&app, Method::POST, &uri, None).await;
            assert_eq!(status, StatusCode::OK, "{res}");
            res
        }
    };

    // Spring 1901: Germany takes Den, France moves next to Bel.
    put(
        "S1901M",
        "GER",
        r#"[{ "power": "GER", "unit": "Kie", "type": "move", "to": "Den" }]"#,
    )
    .await;
    put(
        "S1901M",
        "FRA",
        r#"[{ "power": "FRA", "unit": "Par", "type": "move", "to": "Pic" }]"#,
    )
    .await;
    let res = resolve("S1901M").await;
    assert_eq!(res["nextState"]["phase"], "F1901M");

    // Fall 1901: Germany keeps Den and France takes Bel. Both gain a build, so the
    // adjustment phase follows.
    put(
        "F1901M",
        "FRA",
        r#"[{ "power": "FRA", "unit": "Pic", "type": "move", "to": "Bel" }]"#,
    )
    .await;
    let res = resolve("F1901M").await;
    assert_eq!(res["nextState"]["phase"], "W1901A");
    assert_eq!(res["nextState"]["supplyCenters"]["Den"], "GER");
    assert_eq!(res["nextState"]["supplyCenters"]["Bel"], "FRA");

    put(
        "W1901A",
        "GER",
        r#"[{ "power": "GER", "unit": "Kie", "type": "build", "unitType": "F" }]"#,
    )
    .await;
    put(
        "W1901A",
        "FRA",
        r#"[{ "power": "FRA", "unit": "Par", "type": "build", "unitType": "A" }]"#,
    )
    .await;
    let res = resolve("W1901A").await;
    assert_eq!(res["nextState"]["phase"], "S1902M");
    let units = res["nextState"]["units"].as_array().unwrap();
    assert_eq!(units.len(), 24);
    assert!(units.iter().any(|u| u["location"] == "Kie" && u["type"] == "F"));
    let steps: Vec<&str> = res["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["step"].as_str().unwrap())
        .collect();
    assert_eq!(steps.last(), Some(&"result"));

    // 1902: Germany dislodges the French army in Bel, which retreats to Pic; after the Fall
    // retreat phase SC ownership is updated.
    put(
        "S1902M",
        "GER",
        r#"[
      { "power": "GER", "unit": "Mun", "type": "move", "to": "Ruh" },
      { "power": "GER", "unit": "Ber", "type": "move", "to": "Kie" },
      { "power": "GER", "unit": "Kie", "type": "move", "to": "Hol" }
    ]"#,
    )
    .await;
    let res = resolve("S1902M").await;
    assert_eq!(res["nextState"]["phase"], "F1902M");
    put(
        "F1902M",
        "GER",
        r#"[
      { "power": "GER", "unit": "Ruh", "type": "move", "to": "Bel" },
      { "power": "GER", "unit": "Hol", "type": "support", "target": "Ruh", "to": "Bel" }
    ]"#,
    )
    .await;
    let res = resolve("F1902M").await;
    assert_eq!(res["nextState"]["phase"], "F1902R");
    assert_eq!(res["nextState"]["dislodged"][0]["location"], "Bel");
    put(
        "F1902R",
        "FRA",
        r#"[{ "power": "FRA", "unit": "Bel", "type": "retreat", "to": "Pic" }]"#,
    )
    .await;
    let res = resolve("F1902R").await;
    let steps: Vec<&str> = res["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["step"].as_str().unwrap())
        .collect();
    assert!(
        steps.contains(&"sc-ownership") && steps.contains(&"victory-check"),
        "{steps:?}"
    );
    assert_eq!(res["nextState"]["supplyCenters"]["Bel"], "GER");
    assert_eq!(res["nextState"]["phase"], "W1902A");
}

#[tokio::test]
async fn resolve_with_engine_returns_the_movement_resolution() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path(), Arc::new(Engine));
    let id = new_game(&app).await;
    let rus = r#"[{ "power": "RUS", "unit": "Sev", "type": "move", "to": "Bla" }]"#;
    let tur = r#"[{ "power": "TUR", "unit": "Ank", "type": "move", "to": "Bla" }]"#;
    for (power, body) in [("GER", GERMAN_ORDERS), ("RUS", rus), ("TUR", tur)] {
        let uri = format!("/games/{id}/phases/S1901M/orders/{power}");
        let (status, body) = call(&app, Method::PUT, &uri, Some(body.into())).await;
        assert!(status.is_success(), "{body}");
    }

    let (status, res) = call(&app, Method::POST, &format!("/games/{id}/phases/S1901M/resolve"), None).await;
    assert_eq!(status, StatusCode::OK, "{res}");
    assert_eq!(res["phase"], "S1901M");
    assert_eq!(res["nextState"]["phase"], "F1901M");
    let outcome = |unit: &str| {
        res["orders"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["order"]["unit"] == unit)
            .unwrap()["outcome"]
            .clone()
    };
    assert_eq!(outcome("Ber"), "success");
    assert_eq!(outcome("Sev"), "fail");
    assert_eq!(outcome("Ank"), "fail");
    let units = res["nextState"]["units"].as_array().unwrap();
    assert!(units.iter().any(|u| u["location"] == "Den" && u["power"] == "GER"));
    assert!(
        units.iter().any(|u| u["location"] == "Sev"),
        "the standoff leaves F Sev home"
    );
    let steps: Vec<&str> = res["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["step"].as_str().unwrap())
        .collect();
    for step in ["validate", "strengths", "moves", "result"] {
        assert!(steps.contains(&step), "{step} missing from {steps:?}");
    }
    assert_eq!(read_save(dir.path(), &id)["state"]["phase"], "F1901M");
}

#[tokio::test]
async fn resolved_phases_are_saved_and_replayable() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path(), Arc::new(FakeEngine));
    let id = new_game(&app).await;

    call(
        &app,
        Method::PUT,
        &format!("/games/{id}/phases/S1901M/orders/GER"),
        Some(GERMAN_ORDERS.into()),
    )
    .await;
    let (status, res) = call(&app, Method::POST, &format!("/games/{id}/phases/S1901M/resolve"), None).await;
    assert_eq!(status, StatusCode::OK, "{res}");
    assert_eq!(res["nextState"]["phase"], "F1901M");
    for phase in ["F1901M", "W1901A"] {
        let (status, _) = call(&app, Method::POST, &format!("/games/{id}/phases/{phase}/resolve"), None).await;
        assert_eq!(status, StatusCode::OK);
    }
    // Resolving an old phase again is refused.
    let (status, _) = call(&app, Method::POST, &format!("/games/{id}/phases/S1901M/resolve"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);

    let save = read_save(dir.path(), &id);
    assert_eq!(save["state"]["phase"], "S1902M");
    assert_eq!(save["history"].as_array().unwrap().len(), 3);
    assert_eq!(
        save["orders"],
        json!({}),
        "orders move into the history when a phase is resolved"
    );

    // Replay (after a restart): one frame per resolved phase plus the current position.
    let app = self::app(dir.path(), Arc::new(Engine));
    let (status, replay) = call(&app, Method::GET, &format!("/games/{id}/replay"), None).await;
    assert_eq!(status, StatusCode::OK);
    let frames = replay["frames"].as_array().unwrap();
    let phases: Vec<&str> = frames.iter().map(|f| f["phase"].as_str().unwrap()).collect();
    assert_eq!(phases, ["S1901M", "F1901M", "W1901A", "S1902M"]);
    assert_eq!(frames[0]["orders"]["GER"].as_array().unwrap().len(), 3);
    assert_eq!(frames[0]["resolution"]["trace"][0]["step"], "fake");
    assert!(frames[3].get("resolution").is_none());

    let (status, frame) = call(&app, Method::GET, &format!("/games/{id}/replay/1"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        (frame["total"].as_u64(), frame["frame"]["phase"].as_str()),
        (Some(4), Some("F1901M"))
    );
    let (status, _) = call(&app, Method::GET, &format!("/games/{id}/replay/4"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, history) = call(&app, Method::GET, &format!("/games/{id}/history"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(history["history"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn export_and_import_restore_the_same_game() {
    let src = tempfile::tempdir().unwrap();
    let app_a = app(src.path(), Arc::new(FakeEngine));
    let id = new_game(&app_a).await;
    call(
        &app_a,
        Method::PUT,
        &format!("/games/{id}/phases/S1901M/orders/GER"),
        Some(GERMAN_ORDERS.into()),
    )
    .await;
    call(
        &app_a,
        Method::POST,
        &format!("/games/{id}/phases/S1901M/resolve"),
        None,
    )
    .await;

    let res = app_a
        .clone()
        .oneshot(Request::get(format!("/games/{id}/save")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(
        res.headers()["content-disposition"]
            .to_str()
            .unwrap()
            .contains(&format!("{id}.json"))
    );
    let exported = String::from_utf8(res.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap();
    let original: Game = serde_json::from_str(&exported).unwrap();

    // Import into another server: the restored game is identical.
    let dst = tempfile::tempdir().unwrap();
    let app_b = app(dst.path(), Arc::new(FakeEngine));
    let (status, body) = call(&app_b, Method::POST, "/games/import", Some(exported.clone())).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let restored = GameStore::open(dst.path()).unwrap().load(&id).unwrap();
    assert_eq!(restored, original);

    // Play continues from the restored position.
    let (status, res) = call(
        &app_b,
        Method::POST,
        &format!("/games/{id}/phases/F1901M/resolve"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["nextState"]["phase"], "W1901A");

    // Same ID again → conflict; asNew → a copy under a new ID.
    let (status, _) = call(&app_b, Method::POST, "/games/import", Some(exported.clone())).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, copy) = call(&app_b, Method::POST, "/games/import?asNew=true", Some(exported)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_ne!(copy["gameId"], id.as_str());
    assert_eq!(copy["resolvedPhases"], 1);
    assert_eq!(json_files(dst.path()).len(), 2);
}

#[tokio::test]
async fn broken_save_data_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path(), Arc::new(FakeEngine));
    let id = new_game(&app).await;
    call(&app, Method::POST, &format!("/games/{id}/phases/S1901M/resolve"), None).await;
    let good = read_save(dir.path(), &id);
    let other = tempfile::tempdir().unwrap();
    let import_app = self::app(other.path(), Arc::new(FakeEngine));

    let mut tampered = good.clone();
    tampered["state"]["units"][0]["location"] = json!("Mos"); // no longer matches the recorded result
    let mut bad_version = good.clone();
    bad_version["formatVersion"] = json!(99);
    let mut bad_id = good.clone();
    bad_id["gameId"] = json!("../evil");
    let mut stacked = good.clone();
    stacked["state"]["units"][1]["location"] = stacked["state"]["units"][0]["location"].clone();

    for (name, body) in [
        ("not json", "{".to_string()),
        ("wrong shape", json!({ "hello": 1 }).to_string()),
        ("tampered", tampered.to_string()),
        ("version", bad_version.to_string()),
        ("id", bad_id.to_string()),
        ("two units", stacked.to_string()),
    ] {
        let (status, res) = call(&import_app, Method::POST, "/games/import", Some(body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{name}: {res}");
    }
    assert!(
        json_files(other.path()).is_empty(),
        "nothing written for rejected imports"
    );

    // A corrupted file on disk shows up as unreadable, and loading it fails cleanly.
    std::fs::write(dir.path().join(format!("{id}.json")), tampered.to_string()).unwrap();
    let (_, list) = call(&app, Method::GET, "/games", None).await;
    assert_eq!(list["games"], json!([]));
    assert_eq!(list["unreadable"][0]["file"], format!("{id}.json"));
    let (status, _) = call(&app, Method::GET, &format!("/games/{id}"), None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn unknown_and_invalid_ids() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path(), Arc::new(Engine));
    assert_eq!(
        call(&app, Method::GET, "/games/g_missing", None).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, Method::GET, "/games/bad.id", None).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&app, Method::GET, "/games/..%2Fetc", None).await.0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn map_endpoint_serves_shared_map() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path(), Arc::new(Engine));
    let (status, map) = call(&app, Method::GET, "/maps/standard", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(map["provinces"].as_array().unwrap().len(), 75);
}
