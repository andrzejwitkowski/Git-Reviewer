use crate::adapters::inbound::http::assets;
use crate::adapters::inbound::http::dto::{
    ClipboardExportRequest, ClipboardExportResponse, ErrorResponse, RepoContextResponse,
    RepoStatusResponse, ReviewResponse,
};
use crate::application::use_cases::build_clipboard_export::{
    BuildClipboardExport, BuildClipboardExportError,
    BuildClipboardExportRequest as ClipboardRequest,
};
use crate::application::use_cases::get_repo_status::GetRepoStatus;
use crate::application::use_cases::load_repo_context::{LoadRepoContext, LoadRepoContextError};
use crate::application::use_cases::load_review::{LoadReview, LoadReviewError, LoadReviewRequest};
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::adapters::outbound::git_cli::GitCliRepository;

#[derive(Clone)]
pub struct HttpAppState {
    repo_path: String,
    git: GitCliRepository,
}

impl HttpAppState {
    pub fn new(repo_path: String, git: GitCliRepository) -> Self {
        Self { repo_path, git }
    }
}

#[derive(Deserialize)]
pub struct ReviewQuery {
    pub base: Option<String>,
    pub expand: Option<String>,
}

pub async fn index() -> Html<&'static str> {
    Html(include_str!("../../../../web/index.html"))
}

pub async fn asset(Path(path): Path<String>) -> Response {
    match assets::get(&path) {
        Some(asset) => {
            let mut response = Response::new(asset.content.into());
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(asset.content_type),
            );
            response
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn repo_context(
    State(state): State<HttpAppState>,
) -> Result<Json<RepoContextResponse>, ApiError> {
    let context = LoadRepoContext::new(&state.git)
        .execute()
        .map_err(ApiError::repo_context)?;

    Ok(Json(RepoContextResponse::from_domain(
        state.repo_path,
        context,
    )))
}

pub async fn review(
    State(state): State<HttpAppState>,
    Query(query): Query<ReviewQuery>,
) -> Result<Json<ReviewResponse>, ApiError> {
    let review = LoadReview::new(&state.git)
        .execute(LoadReviewRequest {
            selected_base: query.base,
            expanded_paths: query
                .expand
                .map(|value| {
                    value
                        .split(',')
                        .filter(|item| !item.is_empty())
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default(),
        })
        .map_err(ApiError::review)?;

    Ok(Json(review.into()))
}

pub async fn status(
    State(state): State<HttpAppState>,
) -> Result<Json<RepoStatusResponse>, ApiError> {
    let status = GetRepoStatus::new(&state.git)
        .execute()
        .map_err(|_| ApiError::status())?;

    Ok(Json(status.into()))
}

pub async fn clipboard_export(
    State(state): State<HttpAppState>,
    Json(request): Json<ClipboardExportRequest>,
) -> Result<Json<ClipboardExportResponse>, ApiError> {
    let (head_sha, comments) = request
        .into_domain()
        .map_err(|_| ApiError::invalid_clipboard_export_request())?;
    let text = BuildClipboardExport::new(&state.git)
        .execute(ClipboardRequest { head_sha, comments })
        .map_err(ApiError::clipboard_export)?;

    Ok(Json(ClipboardExportResponse { text }))
}

pub struct ApiError {
    status: StatusCode,
    message: &'static str,
}

impl ApiError {
    fn repo_context(error: LoadRepoContextError) -> Self {
        match error {
            LoadRepoContextError::MissingRemoteBranches => Self {
                status: StatusCode::BAD_REQUEST,
                message: "repo has no remote branches",
            },
            LoadRepoContextError::GitRepository(_) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "repo context unavailable",
            },
        }
    }

    fn review(error: LoadReviewError) -> Self {
        match error {
            LoadReviewError::InvalidBaseBranch => Self {
                status: StatusCode::BAD_REQUEST,
                message: "invalid base branch",
            },
            LoadReviewError::MissingRemoteBranches => Self {
                status: StatusCode::BAD_REQUEST,
                message: "repo has no remote branches",
            },
            LoadReviewError::GitRepository(_) | LoadReviewError::ParseReview(_) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "review unavailable",
            },
        }
    }

    fn status() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "status unavailable",
        }
    }

    fn clipboard_export(error: BuildClipboardExportError) -> Self {
        match error {
            BuildClipboardExportError::MissingAnchoredPath
            | BuildClipboardExportError::MissingSourceContent => Self {
                status: StatusCode::BAD_REQUEST,
                message: "clipboard export unavailable",
            },
            BuildClipboardExportError::GitRepository(_) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "clipboard export unavailable",
            },
        }
    }

    fn invalid_clipboard_export_request() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: "invalid clipboard export request",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}
