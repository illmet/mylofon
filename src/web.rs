mod accounts;
mod session;

use session::{check_csrf, csrf, require_user, viewer};

use crate::{
    auth::{self, Auth},
    config::Config,
    db::{self, Post, Stats, User},
    observability::HttpRequest,
    views,
};
use axum::{
    Form, Json, Router,
    extract::{DefaultBodyLimit, MatchedPath, Path, Query, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use maud::Markup;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tower_http::{compression::CompressionLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tower_sessions::Session;

const PAGE_SIZE: i64 = 25;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
    pub auth: Arc<Auth>,
    stats: Arc<Mutex<Option<(Instant, Stats)>>>,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        // Restrict new database/WAL/key files to the service user via the deployment umask.
        if let Some(parent) = config
            .key_file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        let pool = db::connect(&config.database).await?;
        let auth = auth::initialize(&pool, &config.key_file).await?;
        Ok(Self {
            pool,
            config,
            auth: Arc::new(auth),
            stats: Arc::new(Mutex::new(None)),
        })
    }

    async fn stats(&self) -> Result<Stats, AppError> {
        let mut cache = self.stats.lock().await;
        if let Some((_, stats)) = cache
            .as_ref()
            .filter(|(created, _)| created.elapsed() < Duration::from_secs(15))
        {
            return Ok(stats.clone());
        }
        let stats = db::stats(&self.pool).await?;
        *cache = Some((Instant::now(), stats.clone()));
        Ok(stats)
    }
}

pub async fn app(state: AppState) -> anyhow::Result<Router> {
    let session_layer = session::layer(&state.pool, state.config.secure).await?;
    Ok(Router::new()
        .route("/", get(home))
        .route("/feed", get(home))
        .merge(accounts::routes())
        .route("/posts", post(create_post))
        .route("/post/{id}", get(post_detail))
        .route("/profile", get(my_profile))
        .route("/animals", get(animals))
        .route("/animals/{slug}", get(animal_profile))
        .route("/settings", get(settings))
        .route("/u/{public_id}", get(legacy_profile))
        .route("/u/{public_id}/feed", get(legacy_profile))
        .route("/api/stats", get(stats))
        .route("/healthz", get(health))
        .route("/assets/{*path}", get(asset))
        .fallback(|| async { AppError::new(StatusCode::NOT_FOUND, "This page has wandered off.") })
        .layer(session_layer)
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(20),
        ))
        .layer(middleware::from_fn_with_state(state.clone(), security))
        .layer(CompressionLayer::new())
        .layer(tower::limit::ConcurrencyLimitLayer::new(256))
        .layer(middleware::from_fn(observe_http))
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}

async fn observe_http(request: Request, next: Next) -> Response {
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str())
        .unwrap_or("unmatched");
    let observation = HttpRequest::start(request.method().as_str(), route);
    let response = next.run(request).await;
    observation.finish(response.status().as_u16());
    response
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    message: &'static str,
}
impl AppError {
    fn new(status: StatusCode, message: &'static str) -> Self {
        Self { status, message }
    }
    fn bad(message: &'static str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }
    fn limited() -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "A little too fast. Please wait a minute and try again.",
        )
    }
    fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(%error, "request failed");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong. Please try again.",
        )
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut response = (
            self.status,
            views::page("A small detour", None, "", views::error_page(self.message)),
        )
            .into_response();
        if self.status == StatusCode::TOO_MANY_REQUESTS {
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
        }
        response
    }
}
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        Self::internal(e)
    }
}
impl From<tower_sessions::session::Error> for AppError {
    fn from(e: tower_sessions::session::Error) -> Self {
        Self::internal(e)
    }
}

async fn security(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let unsafe_method = !matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    );
    let cross_site = request
        .headers()
        .get("sec-fetch-site")
        .is_some_and(|s| s == "cross-site");
    let wrong_origin = request
        .headers()
        .get(header::ORIGIN)
        .is_some_and(|v| v.as_bytes() != state.config.origin.as_bytes());
    let mut response = if unsafe_method && (cross_site || wrong_origin) {
        AppError::new(
            StatusCode::FORBIDDEN,
            "This request came from another site. Return to Mylofon and try again.",
        )
        .into_response()
    } else {
        next.run(request).await
    };
    let headers = response.headers_mut();
    if !headers.contains_key(header::CACHE_CONTROL) {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-store"),
        );
    }
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'none'; script-src 'none'; style-src 'self'; img-src 'self'; connect-src 'none'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'; object-src 'none'"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert(
        header::VARY,
        HeaderValue::from_static("Cookie, Accept-Encoding"),
    );
    if state.config.secure {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000"),
        );
    }
    response
}

fn paginate(mut posts: Vec<Post>) -> (Vec<Post>, Option<i64>) {
    let more = posts.len() > PAGE_SIZE as usize;
    posts.truncate(PAGE_SIZE as usize);
    let next = if more {
        posts.last().map(|p| p.id)
    } else {
        None
    };
    (posts, next)
}
fn page(title: &str, user: Option<&User>, token: &str, content: Markup) -> Response {
    views::page(title, user, token, content).into_response()
}

#[derive(Deserialize, Default)]
struct Pagination {
    before: Option<i64>,
}

async fn home(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<Pagination>,
) -> Result<Response, AppError> {
    let user = viewer(&state, &session).await?;
    let token = if user.is_some() {
        csrf(&session).await?
    } else {
        String::new()
    };
    let (posts, next) = paginate(db::feed(&state.pool, query.before, None, PAGE_SIZE + 1).await?);
    Ok(page(
        "Home",
        user.as_ref(),
        &token,
        views::home(user.as_ref(), &token, &posts, next),
    ))
}

async fn my_profile(State(state): State<AppState>, session: Session) -> Result<Response, AppError> {
    let Some(user) = viewer(&state, &session).await? else {
        return Ok(Redirect::to("/login").into_response());
    };
    Ok(Redirect::to(&format!("/animals/{}", views::animal_slug(user.animal))).into_response())
}

async fn legacy_profile(
    State(state): State<AppState>,
    Path(public_id): Path<String>,
) -> Result<Response, AppError> {
    let profile = db::profile(&state.pool, &public_id)
        .await?
        .ok_or_else(|| AppError::new(StatusCode::NOT_FOUND, "This animal hasn't arrived yet."))?;
    Ok(Redirect::to(&format!("/animals/{}", views::animal_slug(profile.animal))).into_response())
}

async fn animals(State(state): State<AppState>, session: Session) -> Result<Response, AppError> {
    let user = viewer(&state, &session).await?;
    Ok(page(
        "Animals",
        user.as_ref(),
        "",
        views::animals(user.as_ref(), ""),
    ))
}

async fn animal_profile(
    State(state): State<AppState>,
    session: Session,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    let animal = (0..15)
        .find(|index| views::animal_slug(*index) == slug)
        .ok_or_else(|| AppError::new(StatusCode::NOT_FOUND, "This animal could not be found."))?;
    let user = viewer(&state, &session).await?;
    Ok(page(
        "Profile",
        user.as_ref(),
        "",
        views::animal_profile(user.as_ref(), "", animal),
    ))
}

async fn settings(State(state): State<AppState>, session: Session) -> Result<Response, AppError> {
    let user = viewer(&state, &session).await?;
    let token = if user.is_some() {
        csrf(&session).await?
    } else {
        String::new()
    };
    Ok(page(
        "Settings",
        user.as_ref(),
        &token,
        views::settings(user.as_ref(), &token),
    ))
}

async fn post_detail(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let user = viewer(&state, &session).await?;
    let post = db::post(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::new(StatusCode::NOT_FOUND, "This post could not be found."))?;
    Ok(page(
        "Post",
        user.as_ref(),
        "",
        views::post_detail(user.as_ref(), "", &post),
    ))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PostForm {
    csrf: String,
    body: String,
}
async fn create_post(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<PostForm>,
) -> Result<Response, AppError> {
    check_csrf(&session, &form.csrf).await?;
    let user = require_user(&state, &session).await?;
    let body = form.body.trim();
    if body.is_empty()
        || body.chars().count() > 5000
        || body
            .chars()
            .any(|c| c == '\0' || (c.is_control() && !matches!(c, '\n' | '\r' | '\t')))
    {
        return Err(AppError::bad(
            "Posts must contain 1–5,000 characters of text.",
        ));
    }
    if !state
        .auth
        .allow(format!("post:{}", user.id), 30, Duration::from_secs(60))
    {
        return Err(AppError::limited());
    }
    let id = db::create_post(&state.pool, user.id, body).await?;
    Ok(Redirect::to(&format!("/#post-{id}")).into_response())
}

async fn stats(State(state): State<AppState>) -> Result<Response, AppError> {
    let stats = state.stats().await?;
    Ok(([(header::CACHE_CONTROL, "public, max-age=15")], Json(stats)).into_response())
}
async fn health(State(state): State<AppState>) -> Result<&'static str, AppError> {
    db::health(&state.pool).await?;
    Ok("ok")
}

async fn asset(Path(path): Path<String>) -> Response {
    let (content_type, data): (&str, &[u8]) = match path.as_str() {
        "style.css" => (
            "text/css; charset=utf-8",
            include_bytes!("../assets/style.css"),
        ),
        _ => return animal_asset(&path),
    };
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        data,
    )
        .into_response()
}

fn animal_asset(path: &str) -> Response {
    // Explicitly embedded assets: no filesystem path traversal and no runtime asset directory.
    let data: &[u8] = match path {
        "animals/red-panda.svg" => include_bytes!("../assets/animals/red-panda.svg"),
        "animals/axolotl.svg" => include_bytes!("../assets/animals/axolotl.svg"),
        "animals/fennec-fox.svg" => include_bytes!("../assets/animals/fennec-fox.svg"),
        "animals/capybara.svg" => include_bytes!("../assets/animals/capybara.svg"),
        "animals/snow-leopard.svg" => include_bytes!("../assets/animals/snow-leopard.svg"),
        "animals/puffin.svg" => include_bytes!("../assets/animals/puffin.svg"),
        "animals/manta-ray.svg" => include_bytes!("../assets/animals/manta-ray.svg"),
        "animals/pangolin.svg" => include_bytes!("../assets/animals/pangolin.svg"),
        "animals/quokka.svg" => include_bytes!("../assets/animals/quokka.svg"),
        "animals/octopus.svg" => include_bytes!("../assets/animals/octopus.svg"),
        "animals/luna-moth.svg" => include_bytes!("../assets/animals/luna-moth.svg"),
        "animals/okapi.svg" => include_bytes!("../assets/animals/okapi.svg"),
        "animals/sea-otter.svg" => include_bytes!("../assets/animals/sea-otter.svg"),
        "animals/secretary-bird.svg" => include_bytes!("../assets/animals/secretary-bird.svg"),
        "animals/wombat.svg" => include_bytes!("../assets/animals/wombat.svg"),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    (
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        data,
    )
        .into_response()
}
