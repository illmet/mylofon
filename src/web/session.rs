//! Cookie configuration, session lifecycle, request identity and CSRF protection.

use super::{AppError, AppState};
use crate::{
    auth,
    db::{self, User},
    observability::ObservedSessionStore,
};
use axum::http::StatusCode;
use sqlx::SqlitePool;
use std::time::Duration;
use subtle::ConstantTimeEq;
use tower_sessions::{
    Expiry, Session, SessionManagerLayer, cookie::SameSite, session_store::ExpiredDeletion,
};
use tower_sessions_sqlx_store::SqliteStore;

pub(super) async fn layer(
    pool: &SqlitePool,
    secure: bool,
) -> anyhow::Result<SessionManagerLayer<ObservedSessionStore>> {
    let store = ObservedSessionStore::new(SqliteStore::new(pool.clone()));
    store.migrate().await?;
    let cleanup = store.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(error) = cleanup.delete_expired().await {
                tracing::error!(%error, "session cleanup failed");
            }
        }
    });
    Ok(SessionManagerLayer::new(store)
        .with_name(if secure {
            "__Host-mylofon"
        } else {
            "mylofon-dev"
        })
        .with_secure(secure)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_path("/")
        .with_expiry(Expiry::OnInactivity(time::Duration::minutes(30))))
}

pub(super) async fn csrf(session: &Session) -> Result<String, AppError> {
    if let Some(token) = session.get::<String>("csrf").await? {
        return Ok(token);
    }
    let token = auth::random_token();
    session.insert("csrf", &token).await?;
    Ok(token)
}
pub(super) async fn check_csrf(session: &Session, supplied: &str) -> Result<(), AppError> {
    let expected = session.get::<String>("csrf").await?.unwrap_or_default();
    if expected.len() != 64 || !bool::from(expected.as_bytes().ct_eq(supplied.as_bytes())) {
        return Err(AppError::new(
            StatusCode::FORBIDDEN,
            "This form expired. Reload the page and try again.",
        ));
    }
    Ok(())
}
pub(super) async fn viewer(state: &AppState, session: &Session) -> Result<Option<User>, AppError> {
    match session.get::<i64>("user_id").await? {
        Some(id) => Ok(db::user(&state.pool, id).await?),
        None => Ok(None),
    }
}
pub(super) async fn require_user(state: &AppState, session: &Session) -> Result<User, AppError> {
    viewer(state, session)
        .await?
        .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "Log in to write a post."))
}
pub(super) async fn establish(session: &Session, user_id: i64) -> Result<String, AppError> {
    session.flush().await?;
    session.cycle_id().await?;
    session.set_expiry(Some(Expiry::AtDateTime(
        time::OffsetDateTime::now_utc() + time::Duration::days(30),
    )));
    session.insert("user_id", user_id).await?;
    csrf(session).await
}
