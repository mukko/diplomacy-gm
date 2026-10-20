use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use diplomacy_gm_server::api::{AppState, router};
use diplomacy_gm_server::engine::Engine;
use diplomacy_gm_server::store::GameStore;
use tower_http::services::{ServeDir, ServeFile};

const USAGE: &str =
    "usage: diplomacy-gm-server [--data-dir DIR] [--addr HOST:PORT] [--web-dir DIR] [--open | --no-open]

  --data-dir DIR    directory holding one <gameId>.json per game
                    (default: $DIPLOMACY_GM_DATA_DIR, else data/games next to the program on
                    Windows and ./data/games elsewhere)
  --addr HOST:PORT  listen address, loopback only (default: the first free port of
                    127.0.0.1:8080-8089)
  --web-dir DIR     serve the GUI from this directory instead of the copy built into the
                    program (default: $DIPLOMACY_GM_WEB_DIR; build it with `npm run build` in web/)
  --open            open the GUI in the default browser once the server runs (default on Windows)
  --no-open         do not open the browser";

/// The GUI build next to this crate (`<repo>/web/dist`), embedded into release builds so that a
/// single executable is enough (docs/en/architecture.md § 7). Debug builds read it from disk.
#[derive(rust_embed::Embed)]
#[folder = "../web/dist"]
#[allow_missing = true]
struct Gui;

#[tokio::main]
async fn main() {
    let mut data_dir = std::env::var_os("DIPLOMACY_GM_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(default_data_dir);
    let mut addr: Option<SocketAddr> = None;
    let mut web_dir = std::env::var_os("DIPLOMACY_GM_WEB_DIR").map(PathBuf::from);
    let mut open = cfg!(windows);

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| {
            args.next()
                .unwrap_or_else(|| exit_usage(&format!("{name} needs a value")))
        };
        match arg.as_str() {
            "--data-dir" => data_dir = value("--data-dir").into(),
            "--web-dir" => web_dir = Some(value("--web-dir").into()),
            "--addr" => {
                addr = Some(
                    value("--addr")
                        .parse()
                        .unwrap_or_else(|e| exit_usage(&format!("--addr: {e}"))),
                )
            }
            "--open" => open = true,
            "--no-open" => open = false,
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            other => exit_usage(&format!("unknown argument `{other}`")),
        }
    }

    // Local single-GM tool: never listen on a network interface (architecture.md § 6).
    if let Some(a) = addr
        && !a.ip().is_loopback()
    {
        exit_usage(&format!(
            "--addr: {a} is not a loopback address; the server only runs locally (e.g. 127.0.0.1:8080)"
        ));
    }

    let store = GameStore::open(&data_dir)
        .unwrap_or_else(|e| fatal(&format!("cannot open data directory {}: {e}", data_dir.display())));
    let mut app = router(AppState {
        store: Arc::new(store),
        adjudicator: Arc::new(Engine),
    });

    // Serve the GUI for every path the API does not use (architecture.md § 7).
    let gui = match &web_dir {
        Some(dir) if dir.join("index.html").is_file() => {
            app = app.fallback_service(ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html"))));
            Some(dir.display().to_string())
        }
        Some(dir) => fatal(&format!("--web-dir: {} has no index.html", dir.display())),
        None if Gui::get("index.html").is_some() => {
            app = app.fallback(embedded_gui);
            Some("built into the program".to_string())
        }
        None => None,
    };

    let listener = bind(addr).await;
    let addr = listener.local_addr().expect("bound address");
    let url = format!("http://{addr}/");
    println!(
        "diplomacy-gm-server listening on {url} (saves in {})",
        data_dir.display()
    );
    match &gui {
        Some(source) => {
            println!("GUI ({source}): open {url} in a browser");
            println!("ブラウザで {url} を開いてください。終了するには Ctrl+C を押すか、このウィンドウを閉じます。");
        }
        None => println!("GUI not built: run `npm install && npm run build` in web/ (or start with ./start.sh)"),
    }
    if open
        && gui.is_some()
        && let Err(e) = webbrowser::open(&url)
    {
        eprintln!("could not open a browser ({e}); open {url} yourself");
    }
    axum::serve(listener, app).await.expect("server error");
}

/// `data/games` next to the executable on Windows (it is usually started by double-clicking),
/// `./data/games` elsewhere.
fn default_data_dir() -> PathBuf {
    if cfg!(windows)
        && let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from))
    {
        return dir.join("data").join("games");
    }
    PathBuf::from("data/games")
}

/// Bind the requested address, or the first free port of 127.0.0.1:8080-8089.
async fn bind(addr: Option<SocketAddr>) -> tokio::net::TcpListener {
    if let Some(addr) = addr {
        return tokio::net::TcpListener::bind(addr)
            .await
            .unwrap_or_else(|e| fatal(&format!("cannot listen on {addr}: {e}")));
    }
    for port in 8080..=8089 {
        if let Ok(l) = tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            return l;
        }
    }
    fatal("ports 8080-8089 are all in use; choose one with --addr 127.0.0.1:PORT")
}

/// Files of the embedded GUI; unknown paths get `index.html` (single-page app).
async fn embedded_gui(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let (path, file) = match Gui::get(path) {
        Some(f) if !path.is_empty() => (path, f),
        _ => match Gui::get("index.html") {
            Some(f) => ("index.html", f),
            None => return StatusCode::NOT_FOUND.into_response(),
        },
    };
    let cache = if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    (
        [
            (header::CONTENT_TYPE, file.metadata.mimetype().to_string()),
            (header::CACHE_CONTROL, cache.to_string()),
        ],
        Body::from(file.data.into_owned()),
    )
        .into_response()
}

/// Print an error and stop. On Windows, wait for Enter so that a window opened by
/// double-clicking does not close before the message can be read.
fn fatal(msg: &str) -> ! {
    eprintln!("error: {msg}");
    if cfg!(windows) {
        eprintln!("Press Enter to close. / Enter キーで閉じます。");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
    std::process::exit(1);
}

fn exit_usage(msg: &str) -> ! {
    eprintln!("{msg}\n\n{USAGE}");
    std::process::exit(2);
}
