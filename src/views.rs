use crate::db::{Post, User};
use maud::{DOCTYPE, Markup, PreEscaped, html};

const ANIMALS: [(&str, &str); 15] = [
    ("Red panda", "red-panda"),
    ("Axolotl", "axolotl"),
    ("Fennec fox", "fennec-fox"),
    ("Capybara", "capybara"),
    ("Snow leopard", "snow-leopard"),
    ("Puffin", "puffin"),
    ("Manta ray", "manta-ray"),
    ("Pangolin", "pangolin"),
    ("Quokka", "quokka"),
    ("Octopus", "octopus"),
    ("Luna moth", "luna-moth"),
    ("Okapi", "okapi"),
    ("Sea otter", "sea-otter"),
    ("Secretary bird", "secretary-bird"),
    ("Wombat", "wombat"),
];

pub fn animal_name(index: i64) -> &'static str {
    ANIMALS[index.rem_euclid(ANIMALS.len() as i64) as usize].0
}

pub fn animal_slug(index: i64) -> &'static str {
    ANIMALS[index.rem_euclid(ANIMALS.len() as i64) as usize].1
}

fn icon(name: &str) -> Markup {
    let path = match name {
        "arrow" => r#"<path d="M7 17 17 7M7 7h10v10"/>"#,
        "back" => r#"<path d="m12 19-7-7 7-7M5 12h14"/>"#,
        "user" => r#"<circle cx="12" cy="8" r="4"/><path d="M4 21v-2a8 8 0 0 1 16 0v2"/>"#,
        _ => "",
    };
    html! {
        svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.65" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" { (PreEscaped(path)) }
    }
}

fn avatar(animal: i64, class: &str) -> Markup {
    html! {
        img class=(class) src=(format!("/assets/animals/{}.svg", animal_slug(animal))) alt=(animal_name(animal)) width="48" height="48" loading="lazy";
    }
}

fn csrf_field(csrf: &str) -> Markup {
    html! { input type="hidden" name="csrf" value=(csrf); }
}

fn date(timestamp: i64) -> String {
    match time::OffsetDateTime::from_unix_timestamp(timestamp) {
        Ok(value) => {
            let month = value.month().to_string();
            format!("{} {}", &month[..3], value.day())
        }
        Err(_) => "Just now".into(),
    }
}

fn date_full(timestamp: i64) -> String {
    match time::OffsetDateTime::from_unix_timestamp(timestamp) {
        Ok(value) => format!(
            "{} {} {}, {:02}:{:02} UTC",
            value.day(),
            value.month(),
            value.year(),
            value.hour(),
            value.minute()
        ),
        Err(_) => "Unknown date".into(),
    }
}

fn account_menu(user: Option<&User>) -> Markup {
    html! {
        details class="account-menu" {
            summary class="account-menu-toggle" aria-label="Account menu" title="Account menu" {
                @if let Some(user) = user {
                    (avatar(user.animal, "avatar"))
                } @else {
                    span class="avatar avatar-placeholder" { (icon("user")) }
                }
                span class="menu-indicator" aria-hidden="true" { "⌄" }
            }
            nav class="account-dropdown" aria-label="Account navigation" {
                a href="/profile" { "Profile" }
                a href="/animals" { "Animals" }
                a href="/settings" { "Settings" }
            }
        }
    }
}

fn topbar(title: &str, user: Option<&User>) -> Markup {
    html! {
        header class="page-header" {
            a class="back-link" href="/" { (icon("back")) "Feed" }
            h1 { (title) }
            (account_menu(user))
        }
    }
}

pub fn page(title: &str, _user: Option<&User>, _csrf: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="theme-color" content="#080808";
                title { (title) " · Mylofon" }
                link rel="stylesheet" href="/assets/style.css?v=7";
            }
            body {
                a class="skip-link" href="#main" { "Skip to content" }
                main id="main" class="main-column" tabindex="-1" { (content) }
            }
        }
    }
}

fn composer(user: &User, csrf: &str) -> Markup {
    html! {
        section class="composer" id="compose" aria-label="Create a post" {
            (account_menu(Some(user)))
            form class="compose-form" action="/posts" method="post" {
                (csrf_field(csrf))
                label class="sr-only" for="post-body" { "Your post" }
                textarea id="post-body" name="body" rows="1" required aria-describedby="post-limit" placeholder="something..." {}
                span id="post-limit" class="sr-only" { "Maximum 5,000 characters" }
                button class="button button-primary" type="submit" { "Post" }
            }
        }
    }
}

pub fn home(user: Option<&User>, csrf: &str, posts: &[Post], next: Option<i64>) -> Markup {
    html! {
        div class="home-layout" {
            @if let Some(user) = user {
                (composer(user, csrf))
            } @else {
                section class="composer guest-composer" aria-label="Account" {
                    (account_menu(None))
                    a class="button button-primary" href="/login" { "Log in or create an account" }
                }
            }
            @if posts.is_empty() {
                div class="empty-state" { p { "No posts yet." } }
            } @else {
                section id="feed" class="feed" aria-label="Posts" tabindex="0" autofocus {
                    @for post in posts { (post_card(post, false)) }
                    @if let Some(before) = next {
                        nav class="feed-page-end post-preview-card" aria-label="Older posts" {
                            a class="next-page" href=(format!("/?before={before}#feed")) rel="next" { "Next posts" (icon("arrow")) }
                        }
                    }
                }
            }
            nav class="feed-pagination" aria-label="Feed pages" {
                a href="/" { "Jump back to latest" }
            }
        }
    }
}

fn post_card(post: &Post, expanded: bool) -> Markup {
    let url = format!("/post/{}", post.id);
    let profile_url = format!("/animals/{}", animal_slug(post.author_animal));
    let is_long = post.body.chars().count() > 500 || post.body.lines().count() > 8;
    let preview: String = post
        .body
        .chars()
        .take(500)
        .collect::<String>()
        .lines()
        .take(8)
        .collect::<Vec<_>>()
        .join("\n");
    html! {
        article id=(format!("post-{}", post.id)) class=(if expanded { "post-card post-expanded" } else { "post-card post-preview-card" }) {
            a class="post-avatar" href=(profile_url) { (avatar(post.author_animal, "avatar")) }
            div class="post-content" {
                div class="post-meta" {
                    a class="author-name" href=(profile_url) { (animal_name(post.author_animal)) }
                    span class="meta-dot" { "·" }
                    a class="post-date" href=(&url) title=(date_full(post.created_at)) { (date(post.created_at)) }
                }
                @if expanded {
                    div class="post-body" { (&post.body) }
                    p class="expanded-date" { (date_full(post.created_at)) }
                } @else {
                    a class="post-preview-link" href=(&url) title="Open full post" {
                        span class="post-body post-preview" { (preview) @if is_long { "…" } }
                    }
                }
            }
        }
    }
}

pub fn post_detail(user: Option<&User>, _csrf: &str, post: &Post) -> Markup {
    html! {
        (topbar("Post", user))
        (post_card(post, true))
    }
}

pub fn login(csrf: &str, error: Option<&str>) -> Markup {
    html! {
        (topbar("Account", None))
        div class="auth-layout" {
            h2 class="auth-title" { "Log in" }
            form class="login-form" action="/login" method="post" {
                (csrf_field(csrf))
                @if let Some(error) = error { div class="form-error" role="alert" { (error) } }
                label for="account-number" { "Account number" }
                input id="account-number" name="account_number" type="password" inputmode="numeric" autocomplete="current-password" placeholder="0000 0000 0000 0000" maxlength="32" required aria-describedby="account-hint";
                p id="account-hint" class="field-hint" { "16 digits. Spaces are fine." }
                button class="button button-primary button-full" type="submit" { "Log in" }
            }
            section class="auth-card" {
                h3 { "New account" }
                p { "You'll receive a random animal and a private 16-digit key." }
                form action="/register" method="post" {
                    (csrf_field(csrf))
                    button class="button button-secondary button-full" type="submit" { "Create your account" }
                }
            }
            p class="privacy-note" { "Keep your key private. Lost keys cannot be recovered." }
        }
    }
}

pub fn welcome(user: &User, csrf: &str, account_number: &str) -> Markup {
    let grouped = account_number
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|part| part.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ");
    html! {
        (topbar("Save your key", Some(user)))
        div class="auth-layout welcome-layout" {
            div class="welcome-identity" { (avatar(user.animal, "avatar")) h2 class="auth-title" { (animal_name(user.animal)) } }
            section class="key-card" aria-labelledby="key-title" {
                h3 id="key-title" { "Your private account number" }
                p { "This is your login and password." }
                code class="account-key" { (grouped) }
                div class="key-warning" { strong { "Save this number before continuing." } "Store it in your password manager. It is only shown here once. Anyone with it can access your account, and a lost number cannot be recovered." }
            }
            form action="/account/confirm" method="post" {
                (csrf_field(csrf))
                label class="save-key-checkbox" { input type="checkbox" name="saved" value="true" required; span { "I've saved my account number somewhere safe." } }
                button class="button button-primary button-full" type="submit" { "Continue to feed" }
            }
        }
    }
}

pub fn animal_profile(user: Option<&User>, _csrf: &str, animal: i64) -> Markup {
    html! {
        (topbar("Profile", user))
        section class="animal-profile" {
            (avatar(animal, "avatar profile-avatar"))
            div { h2 { (animal_name(animal)) } p class="profile-mood" { "Live mood " span { "Neutral" } } }
        }
        section class="animal-summary" aria-labelledby="summary-title" {
            h2 id="summary-title" { "Summary" }
        }
    }
}

pub fn animals(user: Option<&User>, _csrf: &str) -> Markup {
    html! {
        (topbar("Animals", user))
        div class="section-heading animal-list-heading" { span { "Animal" } span { "Live mood" } }
        ul class="animal-list" {
            @for (index, (name, slug)) in ANIMALS.iter().enumerate() {
                li {
                    a class="animal-row" href=(format!("/animals/{slug}")) {
                        (avatar(index as i64, "avatar"))
                        strong { (name) }
                        span class="mood" { "Neutral" }
                    }
                }
            }
        }
    }
}

pub fn settings(user: Option<&User>, csrf: &str) -> Markup {
    html! {
        (topbar("Settings", user))
        div class="settings-layout" {
            @if let Some(user) = user {
                div class="settings-row" {
                    span { "Animal" }
                    a class="settings-identity" href="/profile" { (avatar(user.animal, "avatar avatar-small")) (animal_name(user.animal)) }
                }
            }
            div class="settings-row" { span { "Appearance" } span class="setting-value" { "Dark" } }
            @if user.is_some() {
                form class="settings-logout" action="/logout" method="post" {
                    (csrf_field(csrf))
                    button class="button button-secondary" type="submit" { "Log out" }
                }
            } @else {
                a class="button button-primary settings-logout" href="/login" { "Log in or create an account" }
            }
        }
    }
}

pub fn error_page(message: &str) -> Markup {
    html! {
        (topbar("Error", None))
        div class="empty-state error-state" { p role="alert" { (message) } a class="button button-secondary" href="/" { "Back to feed" } }
    }
}
