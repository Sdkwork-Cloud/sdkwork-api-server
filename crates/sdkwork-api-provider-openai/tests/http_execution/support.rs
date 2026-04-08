pub(crate) use std::sync::{Arc, Mutex};

pub(crate) use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
pub(crate) use futures_util::StreamExt;
pub(crate) use serde_json::{json, Value};
pub(crate) use tokio::net::TcpListener;

#[derive(Clone, Default)]
pub(crate) struct CaptureState {
    pub(crate) authorization: Arc<Mutex<Option<String>>>,
    pub(crate) body: Arc<Mutex<Option<Value>>>,
    pub(crate) content_type: Arc<Mutex<Option<String>>>,
    pub(crate) raw_body: Arc<Mutex<Option<Vec<u8>>>>,
}