use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use serial_test::serial;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[path = "../support/mod.rs"]
mod support;

mod basic_routes;
mod canonical_routes;
mod native_dynamic;
mod route_support;
mod stateful_core;
mod stateless_relay;
mod upstream_fixtures;
mod usage_canonical_outputs;
mod usage_characters;
mod usage_video_lifecycle;

use route_support::*;
use upstream_fixtures::*;
