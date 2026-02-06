//! Admin web dashboard.
//!
//! Simple Axum-based JSON API for managing the bot from a browser.
//! Endpoints:
//! - GET  /api/orders       — list orders (optional ?status= filter)
//! - GET  /api/orders/:id   — get single order
//! - GET  /api/menu         — current menu from config
//! - POST /api/menu         — update a menu item (hot-reload not supported yet)
//! - GET  /api/vouchers     — list all vouchers
//! - POST /api/vouchers     — create a new voucher
//! - GET  /api/stats        — aggregate statistics

use crate::config::HiveConfig;
use crate::store::{OrderStatus, Store};
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Shared state for Axum handlers.
#[derive(Clone)]
struct AppState {
    config: Arc<HiveConfig>,
    store: Store,
}

/// Embedded dashboard HTML (compiled into binary).
const DASHBOARD_HTML: &str = include_str!("../../static/dashboard.html");

/// Start the dashboard web server.
pub async fn run_dashboard(config: HiveConfig, store: Store) -> Result<()> {
    let state = AppState {
        config: Arc::new(config.clone()),
        store,
    };

    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/api/orders", get(list_orders))
        .route("/api/orders/{id}", get(get_order))
        .route("/api/menu", get(get_menu))
        .route("/api/vouchers", get(list_vouchers).post(create_voucher))
        .route("/api/stats", get(get_stats))
        .route("/api/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.dashboard.port);
    log::info!("🌐 Dashboard running at http://localhost:{}", config.dashboard.port);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ─── Query parameters ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct OrdersQuery {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateVoucherRequest {
    amount: f64,
    #[serde(default)]
    code: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

// ─── Handlers ────────────────────────────────────────────────────

async fn serve_dashboard() -> impl IntoResponse {
    Html(DASHBOARD_HTML)
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "service": "hive-dashboard" }))
}

async fn list_orders(
    State(state): State<AppState>,
    Query(params): Query<OrdersQuery>,
) -> impl IntoResponse {
    let status_filter = params.status.as_deref().map(OrderStatus::from_str);

    match state.store.list_orders(status_filter.as_ref()) {
        Ok(orders) => (StatusCode::OK, Json(serde_json::to_value(orders).unwrap())).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.store.get_order(id) {
        Ok(Some(order)) => (StatusCode::OK, Json(serde_json::to_value(order).unwrap())).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: format!("Order {} not found", id),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn get_menu(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::to_value(&state.config.menu).unwrap())
}

async fn list_vouchers(State(state): State<AppState>) -> impl IntoResponse {
    match state.store.list_vouchers() {
        Ok(vouchers) => (StatusCode::OK, Json(serde_json::to_value(vouchers).unwrap())).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn create_voucher(
    State(state): State<AppState>,
    Json(req): Json<CreateVoucherRequest>,
) -> impl IntoResponse {
    if req.amount <= 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "Amount must be positive".to_string(),
            }),
        )
            .into_response();
    }

    let code = req
        .code
        .unwrap_or_else(|| crate::vouchers::generate_voucher_code());

    match state.store.create_voucher(&code, req.amount) {
        Ok(id) => {
            let response = serde_json::json!({
                "id": id,
                "code": code,
                "amount": req.amount,
            });
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    match state.store.get_stats() {
        Ok(stats) => (StatusCode::OK, Json(serde_json::to_value(stats).unwrap())).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}
