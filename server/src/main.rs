use std::net::SocketAddr;
use std::path::PathBuf;

use rastro_core::game::Config;
use rastro_server::state::AppState;

struct Args {
    port: u16,
    admin_token: String,
    public_url: String,
    static_dir: Option<PathBuf>,
    block_secs: u64,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        port: 20910,
        admin_token: std::env::var("RASTRO_ADMIN_TOKEN").unwrap_or_default(),
        public_url: std::env::var("RASTRO_PUBLIC_URL").unwrap_or_default(),
        static_dir: Some(PathBuf::from("frontend/dist")),
        block_secs: 30,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--port" => args.port = value()?.parse().map_err(|e| format!("--port: {e}"))?,
            "--admin-token" => args.admin_token = value()?,
            "--public-url" => args.public_url = value()?,
            "--static" => args.static_dir = Some(PathBuf::from(value()?)),
            "--no-static" => args.static_dir = None,
            "--block-secs" => {
                args.block_secs = value()?.parse().map_err(|e| format!("--block-secs: {e}"))?
            }
            other => return Err(format!("unknown flag {other}")),
        }
    }
    if args.admin_token.is_empty() {
        return Err("set RASTRO_ADMIN_TOKEN or pass --admin-token".to_owned());
    }
    if args.public_url.is_empty() {
        args.public_url = format!("http://localhost:{}", args.port);
    }
    Ok(args)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rastro_server=info,tower_http=info".into()),
        )
        .init();
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let state = match AppState::new(Config::default(), args.admin_token, args.public_url.clone()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not start the game: {e}");
            std::process::exit(1);
        }
    };
    rastro_server::spawn_clock(state.clone(), args.block_secs);
    let app = rastro_server::app(state, args.static_dir);
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("could not bind {addr}: {e}");
            std::process::exit(1);
        }
    };
    tracing::info!("O Rastro on {addr}, public URL {}", args.public_url);
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("server error: {e}");
    }
}
