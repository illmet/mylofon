//! Account HTTP flow: registration, credential login, confirmation and logout.

use super::{
    AppError, AppState, page,
    session::{check_csrf, csrf, establish, viewer},
};
use crate::{auth, db, views};
use axum::{
    Form, Router,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use rand::{Rng, rngs::OsRng};
use serde::Deserialize;
use std::{net::SocketAddr, time::Duration};
use tower_sessions::Session;
use zeroize::Zeroizing;

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(login))
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/account/confirm", post(confirm))
}

async fn login_page(State(state): State<AppState>, session: Session) -> Result<Response, AppError> {
    if viewer(&state, &session).await?.is_some() {
        return Ok(Redirect::to("/").into_response());
    }
    let token = csrf(&session).await?;
    Ok(page("Welcome", None, &token, views::login(&token, None)))
}

#[derive(Deserialize)]
struct CsrfForm {
    csrf: String,
}
#[derive(Deserialize)]
struct LoginForm {
    csrf: String,
    account_number: String,
}

fn limit_auth(
    state: &AppState,
    peer: SocketAddr,
    headers: &HeaderMap,
    register: bool,
) -> Result<(), AppError> {
    let ip = if state.config.trust_proxy && peer.ip().is_loopback() {
        headers
            .get("x-real-ip")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(peer.ip())
    } else {
        peer.ip()
    };
    let bucket = auth::ip_bucket(ip);
    let allowed = state
        .auth
        .allow(format!("auth:{bucket}"), 10, Duration::from_secs(60))
        && state
            .auth
            .allow("auth:global".into(), 120, Duration::from_secs(60));
    if !allowed {
        return Err(AppError::limited());
    }
    if register
        && (!state
            .auth
            .allow(format!("register:{bucket}"), 5, Duration::from_secs(3600))
            || !state
                .auth
                .allow("register:global".into(), 60, Duration::from_secs(3600)))
    {
        return Err(AppError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Account creation is busy. Please try again in an hour.",
        ));
    }
    Ok(())
}

async fn register(
    State(state): State<AppState>,
    session: Session,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(form): Form<CsrfForm>,
) -> Result<Response, AppError> {
    check_csrf(&session, &form.csrf).await?;
    if viewer(&state, &session).await?.is_some() {
        return Ok(Redirect::to("/").into_response());
    }
    limit_auth(&state, peer, &headers, true)?;
    let permit = state
        .auth
        .hashing
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            crate::observability::rate_limit_rejection("capacity");
            AppError::limited()
        })?;
    let number = auth::account_number();
    let lookup = state.auth.lookup(&number);
    let secret = state.auth.verifier(&number);
    let hash = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        auth::hash_secret(&secret)
    })
    .await
    .map_err(AppError::internal)?
    .map_err(AppError::internal)?;
    // Public IDs are independent random identifiers and contain no credential bits.
    let public_id = auth::random_token()[..12].to_owned();
    let user = db::create_user(
        &state.pool,
        &lookup,
        &hash,
        &public_id,
        OsRng.gen_range(0..15),
    )
    .await?;
    let token = establish(&session, user.id).await?;
    Ok(page(
        "Your new identity",
        Some(&user),
        &token,
        views::welcome(&user, &token, &number),
    ))
}

async fn login(
    State(state): State<AppState>,
    session: Session,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    check_csrf(&session, &form.csrf).await?;
    limit_auth(&state, peer, &headers, false)?;
    let submitted = Zeroizing::new(form.account_number);
    let number = auth::normalize_number(&submitted);
    let lookup = state
        .auth
        .lookup(number.as_deref().map(|s| s.as_str()).unwrap_or("invalid"));
    if !state
        .auth
        .allow(format!("key:{lookup}"), 10, Duration::from_secs(60))
    {
        return Err(AppError::limited());
    }
    let credential = if number.is_some() {
        db::credentials(&state.pool, &lookup).await?
    } else {
        None
    };
    let hash = credential
        .as_ref()
        .map(|c| c.password_hash.clone())
        .unwrap_or_else(|| state.auth.dummy_hash.clone());
    let secret = state
        .auth
        .verifier(number.as_deref().map(|s| s.as_str()).unwrap_or("invalid"));
    let permit = state
        .auth
        .hashing
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            crate::observability::rate_limit_rejection("capacity");
            AppError::limited()
        })?;
    let valid = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        auth::verify_secret(&secret, &hash)
    })
    .await
    .map_err(AppError::internal)?;
    if valid && let Some(credential) = credential {
        establish(&session, credential.user_id).await?;
        return Ok(Redirect::to("/").into_response());
    }
    let token = csrf(&session).await?;
    Ok((
        StatusCode::UNAUTHORIZED,
        views::page(
            "Try again",
            None,
            &token,
            views::login(
                &token,
                Some("That account number wasn't recognised. Check all 16 digits and try again."),
            ),
        ),
    )
        .into_response())
}

async fn logout(session: Session, Form(form): Form<CsrfForm>) -> Result<Response, AppError> {
    check_csrf(&session, &form.csrf).await?;
    session.flush().await?;
    Ok(Redirect::to("/login").into_response())
}
async fn confirm(session: Session, Form(form): Form<CsrfForm>) -> Result<Response, AppError> {
    check_csrf(&session, &form.csrf).await?;
    Ok(Redirect::to("/").into_response())
}
