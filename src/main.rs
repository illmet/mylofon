use axum::{Router, http::header, routing::get};
use mylofon::{AppState, app, config::Config, observability};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.as_slice() == ["--help"] || args.as_slice() == ["-h"] {
        println!(
            "Usage: mylofon [--seed-demo]\n\nWithout arguments, starts the server.\n--seed-demo adds 50 demo posts once, then exits."
        );
        return Ok(());
    }
    anyhow::ensure!(
        args.is_empty() || args.as_slice() == ["--seed-demo"],
        "Usage: mylofon [--seed-demo]"
    );
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mylofon=info,tower_http=warn".into()),
        )
        .init();
    let config = Config::from_env()?;
    let bind = config.bind;
    let metrics_listener = match config.metrics_bind.filter(|_| args.is_empty()) {
        Some(address) => Some(tokio::net::TcpListener::bind(address).await?),
        None => None,
    };
    let metrics_handle = metrics_listener
        .as_ref()
        .map(|_| observability::install())
        .transpose()?;
    let state = AppState::new(config).await?;
    if args.as_slice() == ["--seed-demo"] {
        let report = mylofon::demo::seed(&state.pool).await?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        state.pool.close().await;
        return Ok(());
    }
    let sampler = metrics_handle
        .as_ref()
        .map(|handle| {
            observability::sample(state.pool.clone(), &state.config.database, handle.clone())
        })
        .transpose()?;
    let pool = state.pool.clone();
    let router = app(state).await?;
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(%bind, "Mylofon is listening");
    let (stop, signal) = tokio::sync::watch::channel(false);
    let shutdown_task = tokio::spawn(async move {
        shutdown().await;
        let _ = stop.send(true);
    });
    let app_signal = signal.clone();
    let result = tokio::try_join!(
        async move {
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .with_graceful_shutdown(stopping(app_signal))
            .await
        },
        async move {
            if let (Some(listener), Some(handle)) = (metrics_listener, metrics_handle) {
                tracing::info!(address = %listener.local_addr()?, "Private metrics listener is ready");
                let metrics = Router::new().route(
                    "/metrics",
                    get(move || {
                        let handle = handle.clone();
                        async move {
                            (
                                [
                                    (
                                        header::CONTENT_TYPE,
                                        "text/plain; version=0.0.4; charset=utf-8",
                                    ),
                                    (header::CACHE_CONTROL, "no-store"),
                                ],
                                handle.render(),
                            )
                        }
                    }),
                );
                axum::serve(listener, metrics)
                    .with_graceful_shutdown(stopping(signal))
                    .await?;
            }
            Ok::<(), std::io::Error>(())
        },
    );
    shutdown_task.abort();
    if let Some(sampler) = sampler {
        sampler.abort();
    }
    pool.close().await;
    result?;
    Ok(())
}

async fn stopping(mut signal: tokio::sync::watch::Receiver<bool>) {
    let _ = signal.wait_for(|stopping| *stopping).await;
}

async fn shutdown() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("install SIGTERM handler");
        tokio::select! { _ = tokio::signal::ctrl_c() => {}, _ = terminate.recv() => {} }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
