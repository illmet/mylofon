use std::{net::SocketAddr, path::PathBuf};

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub origin: String,
    pub database: String,
    pub key_file: PathBuf,
    pub secure: bool,
    pub trust_proxy: bool,
    pub metrics_bind: Option<SocketAddr>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let origin =
            std::env::var("MYLOFON_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
        let url = url::Url::parse(&origin)?;
        anyhow::ensure!(
            url.path() == "/"
                && url.query().is_none()
                && url.fragment().is_none()
                && url.username().is_empty()
                && url.password().is_none(),
            "MYLOFON_ORIGIN must contain only scheme, host and port"
        );
        let secure = url.scheme() == "https";
        anyhow::ensure!(
            secure
                || (url.scheme() == "http"
                    && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))),
            "Use HTTPS except for local development"
        );
        let bind: SocketAddr = std::env::var("MYLOFON_BIND")
            .unwrap_or_else(|_| "127.0.0.1:3000".into())
            .parse()?;
        anyhow::ensure!(
            secure || bind.ip().is_loopback(),
            "HTTP development must bind to loopback"
        );
        let trust_proxy = std::env::var("MYLOFON_TRUST_PROXY").is_ok_and(|s| s == "true");
        anyhow::ensure!(
            !trust_proxy || bind.ip().is_loopback(),
            "Trusted proxy mode must bind to loopback"
        );
        Ok(Self {
            bind,
            origin: url.origin().ascii_serialization(),
            secure,
            trust_proxy,
            metrics_bind: metrics_address(std::env::var("MYLOFON_METRICS_BIND").ok().as_deref())?,
            database: std::env::var("MYLOFON_DATABASE")
                .unwrap_or_else(|_| "sqlite://data/mylofon.db".into()),
            key_file: std::env::var_os("MYLOFON_KEY_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|| "data/account.key".into()),
        })
    }
}

fn metrics_address(value: Option<&str>) -> anyhow::Result<Option<SocketAddr>> {
    let Some(value) = value else { return Ok(None) };
    let address: SocketAddr = value.parse()?;
    anyhow::ensure!(
        address.ip().is_loopback(),
        "MYLOFON_METRICS_BIND must use a loopback address"
    );
    Ok(Some(address))
}
