//! Explicit, idempotent sample data for local experiments.

use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

const MARKER: &str = "demo_seed_v1";
const DISABLED_PASSWORD: &str = "demo-account-disabled";

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SeedReport {
    pub users: usize,
    pub posts: usize,
}

/// Add sample posts once, without changing any existing accounts or posts.
///
/// Taking the SQLite write lock before reading the marker also makes simultaneous
/// invocations safe. A failed seed rolls back the marker and every inserted row.
pub async fn seed(pool: &SqlitePool) -> anyhow::Result<SeedReport> {
    let mut transaction = pool.begin().await?;
    let inserted = sqlx::query(
        "INSERT INTO instance_settings (name, value) VALUES (?, '{}') \
         ON CONFLICT (name) DO NOTHING",
    )
    .bind(MARKER)
    .execute(&mut *transaction)
    .await?
    .rows_affected();

    if inserted == 0 {
        let report: String =
            sqlx::query_scalar("SELECT value FROM instance_settings WHERE name = ?")
                .bind(MARKER)
                .fetch_one(&mut *transaction)
                .await?;
        let report = serde_json::from_str(&report)?;
        transaction.commit().await?;
        return Ok(report);
    }

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut users = Vec::with_capacity(15);
    for animal in 0..15_i64 {
        // No account key is created or stored. Even a lookup collision cannot
        // authenticate: this sentinel is deliberately not an Argon2 PHC hash.
        let mut lookup = [0_u8; 32];
        OsRng.fill_bytes(&mut lookup);
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO users (lookup_hash, password_hash, public_id, animal, created_at) \
             VALUES (?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(hex::encode(lookup))
        .bind(DISABLED_PASSWORD)
        .bind(format!("demo-{}", crate::views::animal_slug(animal)))
        .bind(animal)
        .bind(now - 172_800)
        .fetch_one(&mut *transaction)
        .await?;
        users.push(id);
    }

    for (index, &(animal, body)) in POSTS.iter().enumerate() {
        let timestamp = now - 165_600 + index as i64 * 3_240;
        sqlx::query("INSERT INTO posts (author_id, body, created_at) VALUES (?, ?, ?)")
            .bind(users[animal])
            .bind(body)
            .bind(timestamp)
            .execute(&mut *transaction)
            .await?;
    }

    let report = SeedReport {
        users: users.len(),
        posts: POSTS.len(),
    };
    sqlx::query("UPDATE instance_settings SET value = ? WHERE name = ?")
        .bind(serde_json::to_string(&report)?)
        .bind(MARKER)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(report)
}

const POSTS: [(usize, &str); 50] = [
    (
        0,
        "The best branch is the one with afternoon sun and absolutely no reason to get down.",
    ),
    (
        1,
        "There is a tiny snail on the glass. It has been travelling toward the same leaf for an hour. I respect a clear plan.",
    ),
    (
        2,
        "The desert is quiet until you start listening. Then it is a very busy place with excellent acoustics.",
    ),
    (
        3,
        "Sat beside the water for a while. Nothing happened. Would do it again.",
    ),
    (
        4,
        "Fresh snow makes every walk feel like being the first person to open a book.",
    ),
    (5, "Wind report: sideways. Hair report: also sideways."),
    (
        6,
        "Went through the same patch of sunlight twice because it felt nice the first time.",
    ),
    (
        7,
        "A practical question: where do you put all the interesting stones you find? My current system is increasingly impractical.",
    ),
    (
        8,
        "I took the long way home and found a tree I had somehow never noticed. Nothing extraordinary about it: a broad trunk, a few pale marks in the bark, one low branch making a very good seat. But from there you could see a strip of sea between two roofs, and the wind reached you before it reached the street.\n\nI stayed until the light changed. A beetle crossed my foot. Somewhere nearby, someone was washing dishes with the window open.\n\nI keep thinking I need to go further away to find a different day. Sometimes apparently I just need to turn left a little earlier.",
    ),
    (
        9,
        "Opened a jar. Closed it. Opened it again. A useful skill deserves practice.",
    ),
    (
        10,
        "The kitchen light was left on. I have cancelled my other plans.",
    ),
    (
        11,
        "The path looks completely different after rain. Same trees, different room.",
    ),
    (
        12,
        "Found a stone that fits perfectly in my paw. A small but significant improvement to the day.",
    ),
    (
        13,
        "Walked a long way this morning. Forgot what I was thinking about. That might have been the point.",
    ),
    (
        14,
        "A doorway should be just large enough to get through and just small enough to feel like yours.",
    ),
    (0, "Bamboo in the rain is an underrated soundtrack."),
    (
        1,
        "Does anybody else have a favourite pebble, or is this a pond-specific thing?",
    ),
    (
        2,
        "Cool sand before sunrise. That is the whole recommendation.",
    ),
    (
        3,
        "Everyone arrived at the pond with something to say. Twenty minutes later we were all watching a leaf float past.",
    ),
    (
        4,
        "The cloud moved and the whole valley appeared. Very dramatic entrance for a place that was there all along.",
    ),
    (
        5,
        "I landed with a fish and great confidence. I retained the fish.",
    ),
    (
        6,
        "Some days the current is doing most of the work. Let it.",
    ),
    (
        7,
        "Reorganised my collection of leaves by colour. It is now harder to find anything, but the corner looks lovely.",
    ),
    (
        8,
        "Someone left half an apple near the path. I am choosing to interpret this as hospitality.",
    ),
    (
        9,
        "Eight arms and I still put something down and immediately lose it.",
    ),
    (
        10,
        "Tonight the moon has a little ring around it. I have no useful observation beyond: look up.",
    ),
    (
        11,
        "I tried walking without hurrying today, which sounds easy until you notice how often you hurry for no particular reason. At first I kept arriving at the next bend before I had really seen the last one. Then I stopped beside a fallen branch and waited for a drop of water to fall from its end. It took much longer than expected.\n\nAfter that, everything seemed to have its own pace. Ants crossed a root. A bird worked carefully through a patch of bark. The clouds moved quickly, but their shadows moved quietly.\n\nI still got home before the rain. I just remember more of the way there.",
    ),
    (
        12,
        "The water was colder than it looked. This information did not change any of my plans.",
    ),
    (
        13,
        "A surprisingly good day for standing very still and looking at the horizon.",
    ),
    (
        14,
        "Finished a small improvement to the tunnel. Nobody else will notice it. I notice it every time.",
    ),
    (
        0,
        "If a nap runs slightly over, that is simply a thorough nap.",
    ),
    (
        1,
        "The snail has reached the leaf. A major development for everyone following along.",
    ),
    (
        2,
        "Heard footsteps, a beetle, distant wind, and somebody opening a packet. The ears are earning their keep.",
    ),
    (
        3,
        "There is room on this rock for one more, provided nobody needs to be in a hurry.",
    ),
    (
        4,
        "Saw my own tracks on the way back and briefly wondered who else had been here.",
    ),
    (
        5,
        "The sea is three different colours today. Four if you count the part I fell into.",
    ),
    (
        6,
        "A school of tiny fish changed direction all at once. No meeting, no announcement. Just a silver turn.",
    ),
    (
        7,
        "I was going to be productive, but a very good patch of sunlight appeared on the floor.",
    ),
    (
        8,
        "What is one ordinary thing near you that you would miss if it disappeared? Mine is the crooked bench under the fig tree.",
    ),
    (9, "Today's colour: approximately the rock behind me."),
    (
        10,
        "A window, a lamp, and a tiny space between the curtain and the glass. An evening with possibilities.",
    ),
    (
        11,
        "Found a route through the trees that comes out exactly where I started. Keeping that one.",
    ),
    (
        12,
        "Floating on my back while it rains feels like being on the correct side of an umbrella.",
    ),
    (
        13,
        "Long legs are useful until the good seat is under a low branch.",
    ),
    (
        14,
        "There is no such thing as a spare blanket. There is only a blanket whose purpose has not yet become clear.",
    ),
    (
        2,
        "Just before dawn the sand holds yesterday's warmth and the air has already moved on.",
    ),
    (
        9,
        "I have inspected all four corners of this rock. Going back to the first one. It had something.",
    ),
    (
        12,
        "There is a little cove I pass most days. Usually I just check the water and carry on, but this morning the tide was low enough to reveal a shallow pool behind the rocks. Inside it were three small fish, a piece of green glass worn completely smooth, and a crab that clearly considered the place private.\n\nI stayed at the edge. The fish did their tiny circuits. The crab disappeared under a ledge, then came back out when I stopped moving. Nothing needed help. Nothing needed taking home.\n\nThe tide will cover it all again by afternoon. I like that there are whole little places that only exist for a few hours.",
    ),
    (
        10,
        "A quiet night. Even the porch light seems less ambitious.",
    ),
    (
        3,
        "No big thoughts today. Just water, a warm stone, and good company.",
    ),
];
