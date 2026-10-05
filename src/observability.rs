// Bounded, anonymous application metrics. Exporting never queries application tables.
//
// Database counters describe logical operations, including session-store calls;
// they are not physical disk reads or SQL statement counters.

use std::{
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use sqlx::{Sqlite, SqlitePool, pool::PoolConnection, sqlite::SqliteConnectOptions};
use tower_sessions::{
    SessionStore,
    session::{Id, Record},
    session_store::{self, ExpiredDeletion},
};
use tower_sessions_sqlx_store::SqliteStore;

const BUCKETS: &[f64] = &[
    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1., 2.5, 5., 10., 20., 60.,
];

fn builder() -> PrometheusBuilder {
    PrometheusBuilder::new()
        .set_buckets(BUCKETS)
        .expect("positive histogram buckets")
}

/// Install once at executable startup. The caller owns the private HTTP listener.
pub fn install() -> anyhow::Result<PrometheusHandle> {
    let handle = builder().install_recorder()?;
    describe_counter!(
        "mylofon_http_requests_total",
        "Completed or cancelled HTTP requests by bounded route and method"
    );
    describe_histogram!(
        "mylofon_http_request_duration_seconds",
        "HTTP dispatch duration including middleware, excluding network body transfer"
    );
    describe_gauge!(
        "mylofon_http_requests_in_flight",
        "HTTP requests currently dispatching"
    );
    describe_counter!(
        "mylofon_db_operations_total",
        "Logical database operations including sessions; not SQL statements or physical reads"
    );
    describe_histogram!(
        "mylofon_db_operation_duration_seconds",
        "Logical database operation duration including pool acquisition"
    );
    describe_histogram!(
        "mylofon_db_pool_acquire_duration_seconds",
        "Application query pool acquisition duration; session-store acquisitions are opaque"
    );
    describe_gauge!(
        "mylofon_db_pool_connections",
        "Sampled SQLite connection counts by state"
    );
    describe_gauge!(
        "mylofon_db_file_size_bytes",
        "SQLite main database and WAL apparent file size; last successful sample"
    );
    describe_gauge!(
        "mylofon_db_file_sample_success",
        "Whether the latest file-size sample succeeded; absent WAL counts as success"
    );
    describe_gauge!(
        "mylofon_uptime_seconds",
        "Seconds since the application monitoring sampler started"
    );
    describe_gauge!(
        "mylofon_process_start_time_seconds",
        "Application monitoring start time as Unix seconds"
    );
    describe_histogram!(
        "mylofon_auth_hash_duration_seconds",
        "Credential hashing and verification duration"
    );
    describe_gauge!(
        "mylofon_auth_hashes_in_flight",
        "Credential hashes and verifications currently executing"
    );
    describe_counter!(
        "mylofon_rate_limit_rejections_total",
        "Rejected operations grouped by bounded limiter scope"
    );
    gauge!("mylofon_http_requests_in_flight").set(0.);
    gauge!("mylofon_auth_hashes_in_flight").set(0.);
    // Register idle operations without recording invented events or durations.
    for (operation, kind) in [
        ("create_user", "write"),
        ("credentials", "read"),
        ("user", "read"),
        ("profile", "read"),
        ("create_post", "write"),
        ("post", "read"),
        ("feed", "read"),
        ("stats", "read"),
        ("health", "read"),
        ("session_load", "read"),
        ("session_create", "write"),
        ("session_save", "write"),
        ("session_delete", "write"),
        ("session_cleanup", "write"),
    ] {
        counter!("mylofon_db_operations_total", "operation" => operation, "kind" => kind, "outcome" => "success").absolute(0);
        let _ = histogram!("mylofon_db_operation_duration_seconds", "operation" => operation, "kind" => kind);
    }
    for scope in [
        "auth_ip",
        "auth_global",
        "register_ip",
        "register_global",
        "account",
        "post",
        "capacity",
        "other",
    ] {
        counter!("mylofon_rate_limit_rejections_total", "scope" => scope).absolute(0);
    }
    for operation in ["hash", "verify"] {
        let _ = histogram!("mylofon_auth_hash_duration_seconds", "operation" => operation);
    }
    Ok(handle)
}

/// RAII keeps cancellation from leaving the in-flight gauge elevated.
pub struct HttpRequest {
    started: Instant,
    method: &'static str,
    route: &'static str,
    in_flight: metrics::Gauge,
    duration: metrics::Histogram,
    completed: bool,
}

impl HttpRequest {
    pub fn start(method: &str, route: &str) -> Self {
        let method = match method {
            "GET" => "GET",
            "POST" => "POST",
            "PUT" => "PUT",
            "PATCH" => "PATCH",
            "DELETE" => "DELETE",
            "HEAD" => "HEAD",
            "OPTIONS" => "OPTIONS",
            _ => "OTHER",
        };
        // Never accept a user-provided URL, even if a future caller forgets to normalize it.
        let route = match route {
            "/" => "/",
            "/feed" => "/feed",
            "/login" => "/login",
            "/register" => "/register",
            "/logout" => "/logout",
            "/account/confirm" => "/account/confirm",
            "/posts" => "/posts",
            "/post/{id}" => "/post/{id}",
            "/profile" => "/profile",
            "/animals" => "/animals",
            "/animals/{slug}" => "/animals/{slug}",
            "/settings" => "/settings",
            "/u/{public_id}" => "/u/{public_id}",
            "/u/{public_id}/feed" => "/u/{public_id}/feed",
            "/api/stats" => "/api/stats",
            "/healthz" => "/healthz",
            "/assets/{*path}" => "/assets/{*path}",
            _ => "unmatched",
        };
        let in_flight = gauge!("mylofon_http_requests_in_flight");
        in_flight.increment(1.);
        Self {
            started: Instant::now(),
            method,
            route,
            in_flight,
            duration: histogram!("mylofon_http_request_duration_seconds", "method" => method, "route" => route),
            completed: false,
        }
    }

    pub fn finish(mut self, status: u16) {
        let status = if (100..=599).contains(&status) {
            status.to_string()
        } else {
            "other".into()
        };
        counter!("mylofon_http_requests_total", "method" => self.method, "route" => self.route, "status" => status).increment(1);
        self.completed = true;
    }
}

impl Drop for HttpRequest {
    fn drop(&mut self) {
        self.in_flight.decrement(1.);
        self.duration.record(self.started.elapsed().as_secs_f64());
        if !self.completed {
            counter!("mylofon_http_requests_total", "method" => self.method, "route" => self.route, "status" => "cancelled").increment(1);
        }
    }
}

pub(crate) struct DbOperation {
    started: Instant,
    operation: &'static str,
    kind: &'static str,
    outcome: &'static str,
}

impl DbOperation {
    pub(crate) fn start(operation: &'static str, kind: &'static str) -> Self {
        Self {
            started: Instant::now(),
            operation,
            kind,
            outcome: "cancelled",
        }
    }

    pub(crate) fn finish<T>(mut self, result: &Result<T, sqlx::Error>) {
        self.outcome = result.as_ref().map_or_else(sqlx_outcome, |_| "success");
    }

    fn finish_session<T>(mut self, result: &session_store::Result<T>) {
        self.outcome = match result {
            Ok(_) => "success",
            Err(session_store::Error::Backend(_)) => "backend",
            Err(session_store::Error::Encode(_)) => "encode",
            Err(session_store::Error::Decode(_)) => "decode",
        };
    }
}

impl Drop for DbOperation {
    fn drop(&mut self) {
        counter!("mylofon_db_operations_total", "operation" => self.operation, "kind" => self.kind, "outcome" => self.outcome).increment(1);
        histogram!("mylofon_db_operation_duration_seconds", "operation" => self.operation, "kind" => self.kind).record(self.started.elapsed().as_secs_f64());
    }
}

fn sqlx_outcome(error: &sqlx::Error) -> &'static str {
    match error {
        sqlx::Error::PoolTimedOut => "pool_timeout",
        sqlx::Error::Database(error) => {
            // SQLite extended error codes retain the primary code in the low byte.
            match error
                .code()
                .and_then(|code| code.parse::<u32>().ok())
                .map(|code| code & 255)
            {
                Some(5 | 6) => "busy",
                Some(19) => "constraint",
                _ => "error",
            }
        }
        _ => "error",
    }
}

pub(crate) async fn acquire(pool: &SqlitePool) -> Result<PoolConnection<Sqlite>, sqlx::Error> {
    struct Timer(Instant);
    impl Drop for Timer {
        fn drop(&mut self) {
            histogram!("mylofon_db_pool_acquire_duration_seconds")
                .record(self.0.elapsed().as_secs_f64());
        }
    }
    let _timer = Timer(Instant::now());
    pool.acquire().await
}

/// Includes the startup dummy hash; the sampler does not perform hashing itself.
pub(crate) struct HashOperation {
    started: Instant,
    duration: metrics::Histogram,
    running: metrics::Gauge,
}

impl HashOperation {
    pub(crate) fn start(operation: &'static str) -> Self {
        let running = gauge!("mylofon_auth_hashes_in_flight");
        running.increment(1.);
        Self {
            started: Instant::now(),
            duration: histogram!("mylofon_auth_hash_duration_seconds", "operation" => operation),
            running,
        }
    }
}

impl Drop for HashOperation {
    fn drop(&mut self) {
        self.running.decrement(1.);
        self.duration.record(self.started.elapsed().as_secs_f64());
    }
}

/// The raw bucket identifier is inspected locally and is never a metric label.
pub fn rate_limit_rejection(key: &str) {
    let scope = match key {
        "auth:global" => "auth_global",
        "register:global" => "register_global",
        "capacity" => "capacity",
        _ if key.starts_with("auth:") => "auth_ip",
        _ if key.starts_with("register:") => "register_ip",
        _ if key.starts_with("key:") => "account",
        _ if key.starts_with("post:") => "post",
        _ => "other",
    };
    counter!("mylofon_rate_limit_rejections_total", "scope" => scope).increment(1);
}

#[derive(Clone, Debug)]
pub struct ObservedSessionStore(SqliteStore);

impl ObservedSessionStore {
    pub fn new(store: SqliteStore) -> Self {
        Self(store)
    }
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        self.0.migrate().await
    }
}

#[async_trait]
impl SessionStore for ObservedSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let operation = DbOperation::start("session_create", "write");
        let result = self.0.create(record).await;
        operation.finish_session(&result);
        result
    }
    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let operation = DbOperation::start("session_save", "write");
        let result = self.0.save(record).await;
        operation.finish_session(&result);
        result
    }
    async fn load(&self, id: &Id) -> session_store::Result<Option<Record>> {
        let operation = DbOperation::start("session_load", "read");
        let result = self.0.load(id).await;
        operation.finish_session(&result);
        result
    }
    async fn delete(&self, id: &Id) -> session_store::Result<()> {
        let operation = DbOperation::start("session_delete", "write");
        let result = self.0.delete(id).await;
        operation.finish_session(&result);
        result
    }
}

#[async_trait]
impl ExpiredDeletion for ObservedSessionStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        let operation = DbOperation::start("session_cleanup", "write");
        let result = self.0.delete_expired().await;
        operation.finish_session(&result);
        result
    }
}

/// Samples bounded filesystem metadata and pool atomics every five seconds. No SQL,
/// WAL checkpoint, /proc scan, or per-request filesystem access is performed here.
pub fn sample(
    pool: SqlitePool,
    database_url: &str,
    handle: PrometheusHandle,
) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let options = SqliteConnectOptions::from_str(database_url)?;
    let database = options.get_filename().to_owned();
    let memory = database.as_os_str() == ":memory:";
    let wal = PathBuf::from(format!("{}-wal", database.display()));
    let started = Instant::now();
    gauge!("mylofon_process_start_time_seconds").set(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64(),
    );
    Ok(tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if pool.is_closed() {
                break;
            }
            let size = pool.size();
            let idle = (pool.num_idle() as u32).min(size);
            gauge!("mylofon_db_pool_connections", "state" => "idle").set(idle as f64);
            gauge!("mylofon_db_pool_connections", "state" => "in_use").set((size - idle) as f64);
            gauge!("mylofon_db_pool_connections", "state" => "max")
                .set(pool.options().get_max_connections() as f64);
            gauge!("mylofon_uptime_seconds").set(started.elapsed().as_secs_f64());
            if !memory {
                sample_file(&database, "database", false).await;
                sample_file(&wal, "wal", true).await;
            }
            // install_recorder deliberately doesn't spawn an upkeep task.
            handle.run_upkeep();
        }
    }))
}

async fn sample_file(path: &std::path::Path, file: &'static str, missing_is_empty: bool) {
    let size = match tokio::fs::metadata(path).await {
        Ok(metadata) => Some(metadata.len()),
        Err(error) if missing_is_empty && error.kind() == std::io::ErrorKind::NotFound => Some(0),
        Err(_) => None,
    };
    gauge!("mylofon_db_file_sample_success", "file" => file).set(if size.is_some() {
        1.
    } else {
        0.
    });
    if let Some(size) = size {
        gauge!("mylofon_db_file_size_bytes", "file" => file).set(size as f64);
    }
}
