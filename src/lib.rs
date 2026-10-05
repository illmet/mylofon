pub mod auth;
pub mod config;
pub mod db;
pub mod demo;
pub mod observability;
pub mod views;
mod web;

pub use web::{AppState, app};
