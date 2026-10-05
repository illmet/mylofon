use std::{str::FromStr, time::Duration};

use serde::Serialize;
use sqlx::{
    QueryBuilder, Sqlite, SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};

use crate::observability::{self, DbOperation};

type Result<T> = std::result::Result<T, sqlx::Error>;

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub public_id: String,
    pub animal: i64,
    pub created_at: i64,
}

#[derive(sqlx::FromRow)]
pub struct Credentials {
    pub user_id: i64,
    pub password_hash: String,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub author_id: i64,
    pub body: String,
    pub created_at: i64,
    pub author_public_id: String,
    pub author_animal: i64,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct Stats {
    pub users: i64,
    pub posts: i64,
}

/// Open a small WAL pool and apply the embedded migrations before serving.
pub async fn connect(url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(8)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options)
        .await?;
    let version: String = sqlx::query_scalar("SELECT sqlite_version()")
        .fetch_one(&pool)
        .await?;
    let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
    if parts.as_slice() < [3, 53, 0].as_slice() {
        return Err(sqlx::Error::Configuration(Box::new(std::io::Error::other(
            format!(
                "SQLite 3.53.0 or newer is required (found {version}); install a patched system SQLite library, then rebuild"
            ),
        ))));
    }
    migrate(&pool).await?;
    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|error| sqlx::Error::Migrate(Box::new(error)))
}

pub async fn create_user(
    pool: &SqlitePool,
    lookup_hash: &str,
    password_hash: &str,
    public_id: &str,
    animal: i64,
) -> Result<User> {
    let operation = DbOperation::start("create_user", "write");
    let result = async {
        let mut connection = observability::acquire(pool).await?;
        sqlx::query_as(
            "INSERT INTO users (lookup_hash, password_hash, public_id, animal) \
             VALUES (?, ?, ?, ?) RETURNING id, public_id, animal, created_at",
        )
        .bind(lookup_hash)
        .bind(password_hash)
        .bind(public_id)
        .bind(animal)
        .fetch_one(&mut *connection)
        .await
    }
    .await;
    operation.finish(&result);
    result
}

pub async fn credentials(pool: &SqlitePool, lookup_hash: &str) -> Result<Option<Credentials>> {
    let operation = DbOperation::start("credentials", "read");
    let result = async {
        let mut connection = observability::acquire(pool).await?;
        sqlx::query_as("SELECT id AS user_id, password_hash FROM users WHERE lookup_hash = ?")
            .bind(lookup_hash)
            .fetch_optional(&mut *connection)
            .await
    }
    .await;
    operation.finish(&result);
    result
}

pub async fn user(pool: &SqlitePool, id: i64) -> Result<Option<User>> {
    let operation = DbOperation::start("user", "read");
    let result = async {
        let mut connection = observability::acquire(pool).await?;
        sqlx::query_as("SELECT id, public_id, animal, created_at FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *connection)
            .await
    }
    .await;
    operation.finish(&result);
    result
}

pub async fn profile(pool: &SqlitePool, public_id: &str) -> Result<Option<User>> {
    let operation = DbOperation::start("profile", "read");
    let result = async {
        let mut connection = observability::acquire(pool).await?;
        sqlx::query_as("SELECT id, public_id, animal, created_at FROM users WHERE public_id = ?")
            .bind(public_id)
            .fetch_optional(&mut *connection)
            .await
    }
    .await;
    operation.finish(&result);
    result
}

pub async fn create_post(pool: &SqlitePool, author_id: i64, body: &str) -> Result<i64> {
    let operation = DbOperation::start("create_post", "write");
    let result = async {
        let mut connection = observability::acquire(pool).await?;
        sqlx::query_scalar("INSERT INTO posts (author_id, body) VALUES (?, ?) RETURNING id")
            .bind(author_id)
            .bind(body)
            .fetch_one(&mut *connection)
            .await
    }
    .await;
    operation.finish(&result);
    result
}

fn post_query<'a>() -> QueryBuilder<'a, Sqlite> {
    QueryBuilder::new(
        "SELECT p.id, p.parent_id, p.author_id, p.body, p.created_at, \
         u.public_id AS author_public_id, u.animal AS author_animal \
         FROM posts p JOIN users u ON u.id = p.author_id",
    )
}

pub async fn post(pool: &SqlitePool, id: i64) -> Result<Option<Post>> {
    let operation = DbOperation::start("post", "read");
    let result = async {
        let mut connection = observability::acquire(pool).await?;
        post_query()
            .push(" WHERE p.id = ")
            .push_bind(id)
            .build_query_as()
            .fetch_optional(&mut *connection)
            .await
    }
    .await;
    operation.finish(&result);
    result
}

/// Newest first. The optional cursor is exclusive; pages contain at most 100 rows.
pub async fn feed(
    pool: &SqlitePool,
    before: Option<i64>,
    author: Option<i64>,
    limit: i64,
) -> Result<Vec<Post>> {
    let operation = DbOperation::start("feed", "read");
    let result = async {
        let mut connection = observability::acquire(pool).await?;
        let mut query = post_query();
        query.push(" WHERE p.parent_id IS NULL");
        if let Some(author) = author {
            query.push(" AND p.author_id = ").push_bind(author);
        }
        if let Some(before) = before {
            query.push(" AND p.id < ").push_bind(before);
        }
        query
            .push(" ORDER BY p.id DESC LIMIT ")
            .push_bind(limit.clamp(1, 100))
            .build_query_as()
            .fetch_all(&mut *connection)
            .await
    }
    .await;
    operation.finish(&result);
    result
}

pub async fn stats(pool: &SqlitePool) -> Result<Stats> {
    let operation = DbOperation::start("stats", "read");
    let result = async {
        let mut connection = observability::acquire(pool).await?;
        sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM users) AS users, \
             (SELECT COUNT(*) FROM posts WHERE parent_id IS NULL) AS posts",
        )
        .fetch_one(&mut *connection)
        .await
    }
    .await;
    operation.finish(&result);
    result
}

/// Lightweight readiness query, included in logical read measurements.
pub async fn health(pool: &SqlitePool) -> Result<()> {
    let operation = DbOperation::start("health", "read");
    let result = async {
        let mut connection = observability::acquire(pool).await?;
        sqlx::query("SELECT 1").execute(&mut *connection).await?;
        Ok(())
    }
    .await;
    operation.finish(&result);
    result
}
