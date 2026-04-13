use crate::adapters::inbound::http::handlers;
use crate::adapters::inbound::http::handlers::HttpAppState;
use crate::adapters::outbound::git_cli::GitCliRepository;
use crate::error::AppError;
use axum::routing::{get, post};
use axum::Router;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::path::Path;

pub fn build_app(repo_path: &Path) -> Router {
    let git = GitCliRepository::new(repo_path);

    Router::new()
        .route("/", get(handlers::index))
        .route("/assets/{*path}", get(handlers::asset))
        .route("/api/repo-context", get(handlers::repo_context))
        .route("/api/commits", get(handlers::commits))
        .route("/api/review", get(handlers::review))
        .route("/api/status", get(handlers::status))
        .route("/api/clipboard-export", post(handlers::clipboard_export))
        .with_state(HttpAppState::new(repo_path.display().to_string(), git))
}

pub async fn serve(repo_path: &Path, port: Option<u16>) -> Result<SocketAddr, AppError> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port.unwrap_or(0)))
        .await
        .map_err(AppError::HttpBindFailed)?;
    let address = listener.local_addr().map_err(AppError::HttpBindFailed)?;
    let app = build_app(repo_path);

    println!("http://{address}");
    io::stdout().flush().map_err(AppError::HttpBindFailed)?;

    axum::serve(listener, app)
        .await
        .map_err(AppError::HttpServeFailed)?;

    Ok(address)
}
