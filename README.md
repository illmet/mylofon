# Mylofon

A small, dark-mode social experiment. Rust renders the whole public interface with Maud. The website serves only HTML, CSS, and SVG: no JavaScript, HTMX, WebAssembly, Node build, CDN, or frontend framework. Its content security policy explicitly blocks scripts.

## Run

Install a current stable Rust toolchain, a C compiler, `pkg-config`, and **SQLite 3.53.0 or newer** with development headers, then:

```sh
umask 077
cargo run --release --locked
```

Open **http://localhost:3000**. The first launch creates `data/mylofon.db`, runs migrations, and generates `data/account.key` with owner-only permissions. Keep that key: accounts cannot be recovered without it. The server refuses to start if an existing database's key is missing or incorrect. Runtime assets and migrations are embedded in the binary.

Check `pkg-config --modversion sqlite3` before building. `.cargo/config.toml` links system SQLite because the compatible session-store release otherwise bundles SQLite 3.46, which predates the [WAL-reset corruption fix](https://www.sqlite.org/wal.html#walreset). The application also checks the runtime version. Install a current SQLite development package from your distribution; older LTS distributions may need an updated SQLite package. The deployed machine must provide the same compatible SQLite shared library.

`.env.example` lists settings. The app reads process environment variables; it does not automatically load `.env`. HTTP is allowed only on a loopback bind with a localhost origin. Production requires `MYLOFON_ORIGIN=https://your-domain` and a TLS reverse proxy. Binding to `127.0.0.1` also keeps the database-backed service off the public network.

## Features

- Random 16-digit account number, shown once; no email, password choice, or recovery flow.
- One random animal per account: red panda, axolotl, fennec fox, capybara, snow leopard, puffin, manta ray, pangolin, quokka, octopus, luna moth, okapi, sea otter, secretary bird, or wombat. Each has an original circular SVG avatar.
- A high-contrast central feed with navigation in the composer avatar: Profile, Animals, and Settings. Profile opens the account's animal summary, currently empty, rather than a personal post history. Animals lists all 15 species with a neutral live mood placeholder. Settings contains account information and logout.
- Posts up to 5,000 Unicode scalar values. Equal-height homepage previews open an individual page with the full text. Emoji and line breaks work; HTML is escaped.
- Native CSS scroll snapping presents four or five complete preview cards on desktop, with fewer on smaller screens. The feed receives keyboard focus on entry for native Space/arrow scrolling; form controls retain their usual keyboard behavior. Ordinary links load the next 25 posts; there is no automatic fetching or ever-growing page. Very short or zoomed viewports allow document scrolling to keep controls accessible.
- Likes and replies are disabled, including their write routes. Existing local records remain intact; legacy replies stay out of the homepage. A direct old post URL can still display its text.
- Axum/Tokio, SQLx/SQLite, durable WAL commits, indexed 25-item cursor pages, persistent sessions, graceful shutdown, and `/healthz` database health check.
- Static asset caching, gzip, and a 15-second in-process cache for public `/api/stats`. HTML and account pages use `private, no-store`. Feed reads need no per-viewer reaction queries. Public browsing does not create sessions.

The initial scope intentionally has no follows, ranking, media uploads, notifications, editing/deletion, or moderation dashboard. All posts are public. Animal profiles are pseudonyms, not a promise of network anonymity. Use a private experimental community until you have appropriate moderation and operational policies for your intended audience.

## Demo posts

```sh
cargo run --release --locked -- --seed-demo
```

This explicitly adds 50 posts from all 15 animals to the configured database. Three longer posts demonstrate previews and full post pages. Existing accounts and content remain intact. The seed runs atomically and stores a completion marker, so rerunning it does not duplicate anything, including databases seeded with the earlier replies/likes version. Demo accounts have no usable login credentials. Normal server startup does not seed data automatically.

## Feed behavior and future evaluation

The browser handles movement with `scroll-snap-type: y mandatory`, per-card
`scroll-snap-stop: always`, and CSS smooth scrolling. Reduced-motion preference
turns smooth scrolling off. This aligns cards without scripting; it does not
reassign Space globally or guarantee one physical trackpad gesture equals one
post in every browser. Native HTML autofocus starts keyboard navigation in the
feed; after using another control, return focus to the feed to scroll with Space.
Pagination and post links perform ordinary document navigation. Browser Back can
restore reading position. The centered footer has one “Jump back to latest” link;
“Next posts” appears at the end of each feed batch when older posts are available.

The current order is newest first, using stable post IDs and an exclusive cursor.
No inference API, external model, scoring policy, or background worker is enabled.
`db::feed` is the single place that selects ordered post data; templates only
render that sequence. Future evaluation should happen outside request handlers:

1. Save a post and an evaluation job in the same short SQLite transaction.
2. A worker with bounded concurrency calls the selected API or local model, with
   timeouts/retries; it never holds a database transaction during inference.
3. Persist results keyed by content revision and evaluator/model version. A hash
   of the exact evaluated text and configuration can avoid duplicate work. Stable
   post IDs remain separate from content hashes.
4. Read stored scores when generating the feed. Freeze a ranked browsing order
   (or its version) across pagination so changing scores do not skip/repeat posts.
   Pending or failed evaluations need an explicit display/ranking policy.

Embeddings provide semantic representations, not an inherent quality verdict.
Choose the judging criteria before choosing a model. A local worker needs its own
CPU/RAM/concurrency budget; model requests must not delay reading or posting.
There is no need for a vector database or distributed queue for the first experiment.

A prepared multi-post thread would add an explicit thread ID and position; the
legacy reply `parent_id` is not an ordered authored thread. Detail pages can later
render a whole sequence and an ordinary reply form without scripting the feed.

## Account security

Account handling is separate from feed handlers: `src/web/accounts.rs` owns the
login, registration, confirmation, and logout routes; `src/web/session.rs` owns
cookies, session lifecycle, identity lookup, and CSRF checks. `src/auth.rs` owns
credential cryptography, rate-limit storage, and database/key binding. These
modules run in the same process and share the existing SQLite pool; there is no
additional service or dependency.

Mullvad describes its number as the account credential; its public client exchanges it for an access token. We found no primary-source evidence that HMAC is Mullvad's login protocol. This implementation uses its own explicit scheme:

1. Generate all 16 decimal digits uniformly with the OS cryptographic RNG, retaining leading zeroes (about 53 bits of entropy).
2. Derive an indexed lookup tag with HMAC-SHA-256 and a private 256-bit server key.
3. Derive a separate, domain-separated HMAC verifier, then store its salted Argon2id PHC hash: 64 MiB, three iterations, one lane, 32-byte output. The Argon2 salt is random per account.
4. Login looks up the tag and verifies Argon2 in a blocking worker. Unknown accounts run dummy verification. A two-slot semaphore bounds hashing memory to roughly 128 MiB plus overhead.

Only the keyed tag and salted hash are stored, never the account number. A database-only leak does not provide a direct offline credential test. **If both the database and server key leak, the fast lookup tag allows guessing without paying Argon2's cost.** Argon2 does not erase that tradeoff, and 16 digits have less entropy than a long random token. Protect the key separately from database exports and ordinary backups. Key rotation and credential recovery are not implemented.

`tower-sessions` with its SQLx store is sufficient; account numbers do not need a custom session system. Production uses an opaque `__Host-mylofon` cookie with Secure, HttpOnly, SameSite=Lax and Path=/, no Domain, and a fixed 30-day expiry. Login rotates the session and CSRF token; logout revokes it. Every mutation checks CSRF; cross-site origins are rejected. Anonymous login forms expire after 30 minutes. Expired session records are cleaned hourly.

Login/registration share limits of 10 attempts/minute/IP (IPv6 grouped by /64) and 120/minute/instance. Login additionally limits each keyed account to 10/minute. Registration is capped at 5/hour/IP and 60/hour/instance; posts at 30/minute/account. Counters are bounded in memory and reset on restart; this is designed for one process. When `MYLOFON_TRUST_PROXY=true`, the listener must be loopback and the proxy must overwrite `X-Real-IP`, as in the supplied Caddy configuration. Add proxy-level connection/abuse limits for an exposed deployment.

Sources: [Mullvad account numbers](https://mullvad.net/en/blog/mullvads-account-numbers-get-longer-and-safer), [Mullvad public client](https://github.com/mullvad/mullvadvpn-app/blob/main/mullvad-api/src/rest.rs), [Argon2 RFC](https://www.rfc-editor.org/rfc/rfc9106.html), [tower-sessions](https://docs.rs/tower-sessions/0.14.0/tower_sessions/).

## Capacity and VPS

There is no meaningful fixed maximum registered-user count: post volume, read/write mix, and storage latency dominate. A conservative **initial planning envelope** is **10,000 registered accounts, up to 1 million posts, and 100–300 simultaneously active people** on **4 modern vCPUs, 8 GB RAM, and at least 80 GB local SSD/NVMe**, behind Caddy. Here “active” means roughly one page/action per person every 10 seconds: 10–30 requests/second total, predominantly reads, with only a few writes/second. These are engineering estimates, not a tested VPS capacity guarantee. A 2-vCPU/4-GB machine is adequate for a small experiment; begin with 4/8 if you want headroom. Choose the specification, not an old plan name; [Hetzner's current cloud plans](https://www.hetzner.com/cloud/cost-optimized/) are one option.

Registered accounts are cheap; content is not. A million maximum-length posts need up to 20 GB for UTF-8 bodies alone, before indexes, WAL, and backups. Typical short posts need much less. Keep substantial free disk headroom and backups off-machine. Pagination does not slow with large offsets because it never uses OFFSET. Feed reads join authors without counting reactions or replies.

SQLite permits concurrent readers but [only one writer at a time](https://www.sqlite.org/wal.html). We keep transactions short, use eight pooled connections and a five-second busy timeout, and retain `synchronous=FULL` for durability. The app limits active requests to 256 and handlers to 20 seconds; those are protective settings, not a user-capacity claim. Argon2 deliberately makes login slower than feed browsing. Measure your actual mixed traffic through HTTPS on the target VPS and aim for p95 below 200 ms and no database-busy errors. Sustained write contention is the signal to migrate to PostgreSQL; extra SQLite app replicas are not the solution.

## Build checks

```sh
cargo build --release --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
```

Local test suites, benchmarks, and dashboard tooling are intentionally excluded
from this repository via `.gitignore`. They remain available in the development
workspace; the application has no build-time dependency on those files. A clean
checkout builds independently with the Rust toolchain and system SQLite.

## Optional metrics

Set `MYLOFON_METRICS_BIND=127.0.0.1:9101` to expose a separate, loopback-only
Prometheus metrics listener. The public application router never exposes
`/metrics`. Instrumentation is implemented in Rust and records requests, latency,
logical database activity, connection-pool waits, authentication work, and
SQLite/WAL sizes. Labels exclude account/post identifiers, credentials, and text.
The application runs without any dashboard or external monitoring service.

HTML responses remain private and uncached; static assets and public `/api/stats`
have bounded caching. Metrics sampling does not scan post tables or run WAL
maintenance. HTTP timing ends when response headers are ready, before network
body transfer, and logical SQL operations are distinct from physical disk I/O.

## Deploy and back up

Build with `cargo build --release --locked`, create a dedicated `mylofon` system user, and install `target/release/mylofon` at `/opt/mylofon/mylofon`. Install [deploy/mylofon.service](deploy/mylofon.service), set `MYLOFON_ORIGIN=https://your-domain` in `/etc/mylofon.env`, and adapt [deploy/Caddyfile](deploy/Caddyfile). The systemd unit creates private state directories, uses a separate key directory, and applies a restrictive umask. Run one application instance with local storage, not a network filesystem. Expose only ports 80/443 through your firewall.

Back up SQLite with its online backup API/CLI, not by copying only the live `.db` file and ignoring WAL:

```sh
umask 077
sqlite3 /var/lib/mylofon/mylofon.db ".backup '/your/secure-backup/mylofon.db'"
```

Save `/var/lib/mylofon-keys/account.key` in a separately protected encrypted backup. Database backups contain active session data and must also be private. Test restoring both the database and the original key together; preserve `0600` key permissions. Monitor disk usage, response latency, error logs, and `/healthz`. No deployment has been performed by this repository setup.

Application templates, styling, and SVG assets are embedded and have no external network dependencies.
