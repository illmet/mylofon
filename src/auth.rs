use crate::observability;

use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString,
};
use hmac::{Hmac, Mac};
use rand::{Rng, RngCore, rngs::OsRng};
use sha2::Sha256;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::IpAddr,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;
use zeroize::Zeroizing;

pub struct Auth {
    key: Zeroizing<[u8; 32]>,
    pub dummy_hash: String,
    pub hashing: Arc<Semaphore>,
    buckets: Mutex<HashMap<String, (Instant, u32)>>,
}

impl Auth {
    pub fn new(key: [u8; 32]) -> anyhow::Result<Self> {
        let mut auth = Self {
            key: Zeroizing::new(key),
            dummy_hash: String::new(),
            hashing: Arc::new(Semaphore::new(2)),
            buckets: Mutex::new(HashMap::new()),
        };
        auth.dummy_hash = hash_secret(&auth.verifier(&random_token()))?;
        Ok(auth)
    }

    fn keyed(&self, domain: &[u8], number: &str) -> String {
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.key.as_ref()).expect("HMAC accepts 32 byte keys");
        mac.update(domain);
        mac.update(number.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    pub fn lookup(&self, number: &str) -> String {
        self.keyed(b"mylofon:lookup:v1\0", number)
    }
    pub fn fingerprint(&self) -> String {
        self.keyed(b"mylofon:key-fingerprint:v1\0", "")
    }
    pub fn verifier(&self, number: &str) -> Zeroizing<String> {
        Zeroizing::new(self.keyed(b"mylofon:verify:v1\0", number))
    }

    /// Fixed windows; bounded memory, expired entries reclaimed, fail closed when full.
    pub fn allow(&self, key: String, max: u32, window: Duration) -> bool {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap_or_else(|p| p.into_inner());
        if buckets.len() >= 10_000 {
            buckets.retain(|_, (expires, _)| *expires > now);
        }
        if buckets.len() >= 10_000 && !buckets.contains_key(&key) {
            observability::rate_limit_rejection(&key);
            return false;
        }
        let entry = buckets.entry(key.clone()).or_insert((now + window, 0));
        if entry.0 <= now {
            *entry = (now + window, 0);
        }
        if entry.1 >= max {
            observability::rate_limit_rejection(&key);
            return false;
        }
        entry.1 += 1;
        true
    }
}

/// Load the instance key and verify that it belongs to this account database.
pub async fn initialize(pool: &sqlx::SqlitePool, key_file: &Path) -> anyhow::Result<Auth> {
    let has_users: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users)")
        .fetch_one(pool)
        .await?;
    anyhow::ensure!(
        key_file.exists() || !has_users,
        "Account key is missing for an existing database. Restore the original key file from backup."
    );
    let key = load_key(key_file)?;
    let auth = tokio::task::spawn_blocking(move || Auth::new(key)).await??;
    sqlx::query("INSERT INTO instance_settings (name, value) VALUES ('key_fingerprint', ?) ON CONFLICT(name) DO NOTHING")
        .bind(auth.fingerprint()).execute(pool).await?;
    let fingerprint: String =
        sqlx::query_scalar("SELECT value FROM instance_settings WHERE name = 'key_fingerprint'")
            .fetch_one(pool)
            .await?;
    anyhow::ensure!(
        fingerprint == auth.fingerprint(),
        "Account key does not match this database. Restore the original key file from backup."
    );
    Ok(auth)
}

pub fn hash_secret(secret: &str) -> anyhow::Result<String> {
    let _operation = observability::HashOperation::start("hash");
    let params = Params::new(65_536, 3, 1, Some(32)).map_err(|e| anyhow::anyhow!("{e}"))?;
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password(secret.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|hash| hash.to_string())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn verify_secret(secret: &str, encoded: &str) -> bool {
    let _operation = observability::HashOperation::start("verify");
    PasswordHash::new(encoded).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(secret.as_bytes(), &hash)
            .is_ok()
    })
}

pub fn account_number() -> Zeroizing<String> {
    Zeroizing::new(
        (0..16)
            .map(|_| char::from(b'0' + OsRng.gen_range(0..10)))
            .collect(),
    )
}

pub fn normalize_number(input: &str) -> Option<Zeroizing<String>> {
    if input.len() > 64 {
        return None;
    }
    let number: String = input.chars().filter(|c| *c != ' ' && *c != '-').collect();
    (number.len() == 16 && number.bytes().all(|b| b.is_ascii_digit()))
        .then(|| Zeroizing::new(number))
}

pub fn random_token() -> String {
    let mut bytes = [0; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn ip_bucket(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => {
            if let Some(ip) = ip.to_ipv4_mapped() {
                return ip.to_string();
            }
            let s = ip.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
    }
}

pub fn load_key(path: &Path) -> anyhow::Result<[u8; 32]> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(path) {
        Ok(mut file) => {
            let mut key = [0; 32];
            OsRng.fill_bytes(&mut key);
            file.write_all(&key)?;
            file.sync_all()?;
            Ok(key)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                anyhow::ensure!(
                    std::fs::metadata(path)?.permissions().mode() & 0o077 == 0,
                    "Account key is readable by other users; set its permissions to 0600"
                );
            }
            let mut bytes = Zeroizing::new(Vec::new());
            std::fs::File::open(path)?
                .take(33)
                .read_to_end(&mut bytes)?;
            anyhow::ensure!(
                bytes.len() == 32,
                "Account key must be exactly 32 bytes; restore its backup, do not replace it"
            );
            Ok(bytes.as_slice().try_into()?)
        }
        Err(e) => Err(e.into()),
    }
}
