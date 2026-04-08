use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::Value;
use sqlx::SqlitePool;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[path = "../support/mod.rs"]
mod support;

mod basic_routes;
mod route_support;
mod stateful_core;
mod stateless_relay;
mod upstream_fixtures;
mod usage_billing;

use route_support::*;
use upstream_fixtures::*;
