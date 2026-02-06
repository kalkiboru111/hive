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
use crate::payments::{B2CClient, MpesaCallback, process_callback};
use crate::store::{OrderStatus, Store};
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Shared state for Axum handlers.
#[derive(Clone)]
struct AppState {
    config: Arc<HiveConfig>,
    store: Store,
    wa_client: Arc<tokio::sync::RwLock<Option<Arc<whatsapp_rust::client::Client>>>>,
    b2c_client: Option<Arc<B2CClient>>,
}

/// Embedded dashboard HTML (compiled into binary).
const DASHBOARD_HTML: &str = include_str!("../../static/dashboard.html");

/// Start the dashboard web server.
pub async fn run_dashboard(
    config: HiveConfig,
    store: Store,
    wa_client: Arc<tokio::sync::RwLock<Option<Arc<whatsapp_rust::client::Client>>>>,
) -> Result<()> {
    // Initialize B2C client if configured
    let b2c_client = if let Some(ref mpesa_cfg) = config.payments.mpesa {
        // B2C requires additional config beyond STK Push
        // For now, skip B2C initialization (requires separate credentials)
        // TODO: Add b2c config to payments section
        None
    } else {
        None
    };

    let state = AppState {
        config: Arc::new(config.clone()),
        store,
        wa_client,
        b2c_client,
    };

    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/api/orders", get(list_orders))
        .route("/api/orders/{id}", get(get_order))
        .route("/api/menu", get(get_menu))
        .route("/api/vouchers", get(list_vouchers).post(create_voucher))
        .route("/api/stats", get(get_stats))
        .route("/api/health", get(health_check))
        .route("/api/payments", get(list_payments))
        .route("/api/payments/{id}", get(get_payment))
        .route("/api/payments/{id}/refund", post(refund_payment))
        .route("/api/mpesa/callback", post(mpesa_callback))
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

/// M-Pesa webhook handler for payment callbacks
async fn mpesa_callback(
    State(state): State<AppState>,
    Json(callback): Json<MpesaCallback>,
) -> impl IntoResponse {
    log::info!("📥 M-Pesa callback received");
    
    // Get WhatsApp client (may be None if bot not connected yet)
    let wa_client = {
        let client_lock = state.wa_client.read().await;
        client_lock.clone()
    };
    
    match process_callback(callback, &state.store, &state.config, wa_client).await {
        Ok(result) => {
            log::info!("✅ {}", result.message);
            (StatusCode::OK, Json(serde_json::json!({
                "ResultCode": 0,
                "ResultDesc": "Accepted"
            }))).into_response()
        }
        Err(e) => {
            log::error!("❌ M-Pesa callback processing failed: {}", e);
            (StatusCode::OK, Json(serde_json::json!({
                "ResultCode": 1,
                "ResultDesc": format!("Error: {}", e)
            }))).into_response()
        }
    }
}

/// List all payments with optional filtering
async fn list_payments(State(state): State<AppState>) -> impl IntoResponse {
    // For now, get all payments by querying each order
    // TODO: Add direct payments query to store
    match state.store.list_orders(None) {
        Ok(orders) => {
            let mut all_payments = Vec::new();
            for order in orders {
                if let Ok(payments) = state.store.get_order_payments(order.id) {
                    all_payments.extend(payments);
                }
            }
            
            // Sort by created_at descending
            all_payments.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            
            (StatusCode::OK, Json(all_payments)).into_response()
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

/// Get single payment by ID
async fn get_payment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.store.get_payment(&id) {
        Ok(Some(payment)) => (StatusCode::OK, Json(payment)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiError {
            error: "Payment not found".to_string(),
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Refund a completed payment
async fn refund_payment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Check if B2C is configured
    let b2c = match state.b2c_client.as_ref() {
        Some(client) => client,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError {
                    error: "M-Pesa B2C (refunds) not configured. Contact admin.".to_string(),
                }),
            ).into_response();
        }
    };

    // Get payment
    let payment = match state.store.get_payment(&id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(ApiError {
                error: "Payment not found".to_string(),
            })).into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: e.to_string(),
                }),
            ).into_response();
        }
    };

    // Check if payment is completed
    if !matches!(payment.status, crate::payments::PaymentStatus::Completed) {
        return (StatusCode::BAD_REQUEST, Json(ApiError {
            error: "Can only refund completed payments".to_string(),
        })).into_response();
    }

    // Initiate B2C refund
    match b2c.refund_payment(payment.amount, &payment.phone, payment.order_id).await {
        Ok(conversation_id) => {
            log::info!("💸 Refund initiated for payment {}: ConversationID={}", payment.id, conversation_id);
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "conversation_id": conversation_id,
                "message": format!("Refund of {}{} initiated to {}", 
                                  state.config.business.currency, 
                                  payment.amount, 
                                  payment.phone)
            }))).into_response()
        }
        Err(e) => {
            log::error!("❌ Refund failed for payment {}: {}", payment.id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Refund failed: {}", e),
                }),
            ).into_response()
        }
    }
}
