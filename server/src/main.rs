use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use diplomacy_gm_server::api::{AppState, router};
use diplomacy_gm_server::engine::Engine;
use diplomacy_gm_server::store::GameStore;

const USAGE: &str = "usage: diplomacy-gm-server [--data-dir DIR] [--addr HOST:PORT]

  --data-dir DIR    directory holding one <gameId>.json per game
                    (default: $DIPLOMACY_GM_DATA_DIR or ./data/games)
  --addr HOST:PORT  listen address, loopback only (default: 127.0.0.1:8080)";

#[tokio::main]
async fn main() {
    let mut data_dir = std::env::var_os("DIPLOMACY_GM_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/games"));
    let mut addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| {
            args.next()
                .unwrap_or_else(|| exit_usage(&format!("{name} needs a value")))
        };
        match arg.as_str() {
            "--data-dir" => data_dir = value("--data-dir").into(),
            "--addr" => {
                addr = value("--addr")
                    .parse()
                    .unwrap_or_else(|e| exit_usage(&format!("--addr: {e}")))
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            other => exit_usage(&format!("unknown argument `{other}`")),
        }
    }

    // Local single-GM tool: never listen on a network interface (architecture.md § 6).
    if !addr.ip().is_loopback() {
        exit_usage(&format!(
            "--addr: {addr} is not a loopback address; the server only runs locally (e.g. 127.0.0.1:8080)"
        ));
    }

    let store = GameStore::open(&data_dir).unwrap_or_else(|e| {
        eprintln!("cannot open data directory {}: {e}", data_dir.display());
        std::process::exit(1);
    });
    let app = router(AppState {
        store: Arc::new(store),
        adjudicator: Arc::new(Engine),
    });

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|e| {
        eprintln!("cannot listen on {addr}: {e}");
        std::process::exit(1);
    });
    println!(
        "diplomacy-gm-server listening on http://{addr} (saves in {})",
        data_dir.display()
    );
    axum::serve(listener, app).await.expect("server error");
}

fn exit_usage(msg: &str) -> ! {
    eprintln!("{msg}\n\n{USAGE}");
    std::process::exit(2);
}
