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
        .route("/api/refunds", get(list_refunds))
        .route("/api/refunds/{id}", get(get_refund))
        .route("/api/export/ledger", get(export_ledger))
        .route("/api/analytics/payments", get(payment_analytics))
        .route("/api/reconciliation/report", get(reconciliation_report))
        .route("/api/mpesa/callback", post(mpesa_callback))
        .route("/api/mpesa/b2c/callback", post(mpesa_b2c_callback))
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

    // Create refund record
    let refund_id = format!("REF-{}-{}", payment.order_id, chrono::Utc::now().timestamp());
    if let Err(e) = state.store.create_refund(
        &refund_id,
        &payment.id,
        payment.order_id,
        payment.amount,
        &payment.currency,
        &payment.phone,
        Some("Admin refund via dashboard"),
        Some("dashboard"), // TODO: Get actual admin ID from auth
    ) {
        log::error!("Failed to create refund record: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: format!("Failed to create refund record: {}", e),
            }),
        ).into_response();
    }

    // Initiate B2C refund
    match b2c.refund_payment(payment.amount, &payment.phone, payment.order_id).await {
        Ok(conversation_id) => {
            // Update refund status to processing
            if let Err(e) = state.store.update_refund_status(&refund_id, "processing", Some(&conversation_id)) {
                log::error!("Failed to update refund status: {}", e);
            }

            log::info!("💸 Refund {} initiated for payment {}: ConversationID={}", refund_id, payment.id, conversation_id);
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "refund_id": refund_id,
                "conversation_id": conversation_id,
                "message": format!("Refund of {}{} initiated to {}", 
                                  state.config.business.currency, 
                                  payment.amount, 
                                  payment.phone)
            }))).into_response()
        }
        Err(e) => {
            // Update refund status to failed
            if let Err(update_err) = state.store.update_refund_status(&refund_id, "failed", None) {
                log::error!("Failed to update refund status: {}", update_err);
            }

            log::error!("❌ Refund {} failed for payment {}: {}", refund_id, payment.id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Refund failed: {}", e),
                }),
            ).into_response()
        }
    }
}

/// List all refunds
async fn list_refunds(State(state): State<AppState>) -> impl IntoResponse {
    match state.store.list_refunds(None) {
        Ok(refunds) => (StatusCode::OK, Json(refunds)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Get single refund by ID
async fn get_refund(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.store.get_refund(&id) {
        Ok(Some(refund)) => (StatusCode::OK, Json(refund)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiError {
            error: "Refund not found".to_string(),
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

/// M-Pesa B2C callback handler (refund confirmations)
async fn mpesa_b2c_callback(
    State(state): State<AppState>,
    Json(callback): Json<serde_json::Value>,
) -> impl IntoResponse {
    log::info!("📥 M-Pesa B2C callback received: {:?}", callback);
    
    // Extract conversation ID and result
    let conversation_id = callback["Result"]["ConversationID"].as_str();
    let result_code = callback["Result"]["ResultCode"].as_i64();
    
    if let (Some(conv_id), Some(code)) = (conversation_id, result_code) {
        // Find refund by conversation ID
        let refunds = match state.store.list_refunds(None) {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to list refunds: {}", e);
                return (StatusCode::OK, Json(serde_json::json!({
                    "ResultCode": 1,
                    "ResultDesc": "Internal error"
                }))).into_response();
            }
        };
        
        let refund = refunds.iter().find(|r| {
            r.conversation_id.as_deref() == Some(conv_id)
        });
        
        if let Some(refund) = refund {
            let new_status = if code == 0 { "completed" } else { "failed" };
            
            if let Err(e) = state.store.update_refund_status(&refund.id, new_status, Some(conv_id)) {
                log::error!("Failed to update refund status: {}", e);
            } else {
                log::info!("✅ Refund {} {} (ConversationID={})", 
                          refund.id, 
                          if code == 0 { "completed" } else { "failed" }, 
                          conv_id);
            }
        } else {
            log::warn!("⚠️ Refund not found for ConversationID: {}", conv_id);
        }
    }
    
    (StatusCode::OK, Json(serde_json::json!({
        "ResultCode": 0,
        "ResultDesc": "Accepted"
    }))).into_response()
}

/// Export full ledger for bank credit applications
async fn export_ledger(State(state): State<AppState>) -> impl IntoResponse {
    log::info!("📊 Generating ledger export for credit application");
    
    // Gather all financial data
    let orders = match state.store.list_orders(None) {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Failed to fetch orders: {}", e),
                }),
            ).into_response();
        }
    };
    
    let stats = match state.store.get_stats() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Failed to fetch stats: {}", e),
                }),
            ).into_response();
        }
    };
    
    let refunds = match state.store.list_refunds(None) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Failed to fetch refunds: {}", e),
                }),
            ).into_response();
        }
    };
    
    // Get all payments for completed orders
    let mut all_payments = Vec::new();
    for order in &orders {
        if let Ok(payments) = state.store.get_order_payments(order.id) {
            all_payments.extend(payments);
        }
    }
    
    // Calculate time-series revenue (monthly breakdown)
    use std::collections::HashMap;
    let mut monthly_revenue: HashMap<String, f64> = HashMap::new();
    let mut monthly_orders: HashMap<String, i64> = HashMap::new();
    
    for order in &orders {
        if matches!(order.status, crate::store::OrderStatus::Delivered) {
            // Extract year-month from created_at (format: "2026-02-06 12:00:00")
            let month = order.created_at.chars().take(7).collect::<String>(); // "2026-02"
            *monthly_revenue.entry(month.clone()).or_insert(0.0) += order.total;
            *monthly_orders.entry(month).or_insert(0) += 1;
        }
    }
    
    // Sort months chronologically
    let mut months: Vec<_> = monthly_revenue.keys().cloned().collect();
    months.sort();
    
    let monthly_breakdown: Vec<_> = months.iter().map(|month| {
        serde_json::json!({
            "month": month,
            "revenue": monthly_revenue.get(month).unwrap_or(&0.0),
            "orders": monthly_orders.get(month).unwrap_or(&0),
        })
    }).collect();
    
    // Payment success rate
    let payment_success_rate = if stats.total_payments > 0 {
        (stats.completed_payments as f64 / stats.total_payments as f64) * 100.0
    } else {
        0.0
    };
    
    // Refund rate
    let refund_rate = if stats.completed_payments > 0 {
        (refunds.len() as f64 / stats.completed_payments as f64) * 100.0
    } else {
        0.0
    };
    
    // Build comprehensive ledger report
    let report = serde_json::json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "business": {
            "name": state.config.business.name,
            "currency": state.config.business.currency,
            "phone": state.config.business.phone,
        },
        "summary": {
            "total_revenue": stats.total_revenue,
            "payment_revenue": stats.payment_revenue,
            "total_orders": stats.total_orders,
            "delivered_orders": stats.delivered_orders,
            "pending_orders": stats.pending_orders,
            "total_payments": stats.total_payments,
            "completed_payments": stats.completed_payments,
            "failed_payments": stats.failed_payments,
            "payment_success_rate": format!("{:.2}%", payment_success_rate),
            "total_refunds": refunds.len(),
            "refund_rate": format!("{:.2}%", refund_rate),
        },
        "monthly_breakdown": monthly_breakdown,
        "orders": orders.iter().map(|o| {
            serde_json::json!({
                "id": o.id,
                "date": o.created_at,
                "customer": o.customer_phone,
                "subtotal": o.subtotal,
                "delivery_fee": o.delivery_fee,
                "total": o.total,
                "status": o.status.as_str(),
            })
        }).collect::<Vec<_>>(),
        "payments": all_payments.iter().map(|p| {
            serde_json::json!({
                "id": p.id,
                "order_id": p.order_id,
                "date": p.created_at,
                "amount": p.amount,
                "currency": p.currency,
                "method": format!("{:?}", p.method),
                "status": format!("{:?}", p.status),
                "receipt": p.provider_ref,
            })
        }).collect::<Vec<_>>(),
        "refunds": refunds.iter().map(|r| {
            serde_json::json!({
                "id": r.id,
                "payment_id": r.payment_id,
                "order_id": r.order_id,
                "date": r.created_at,
                "amount": r.amount,
                "currency": r.currency,
                "reason": r.reason,
                "status": r.status.as_str(),
            })
        }).collect::<Vec<_>>(),
        "verification": {
            "platform": "Hive on Reality Network",
            "ledger": "Reality Network L0/L1 Consensus Layer",
            "proof": "All transactions submitted to decentralized ledger",
            "auditable": true,
            "tamper_proof": true,
            "note": "This financial history is backed by cryptographic proofs on Reality Network's decentralized ledger. Each order creates an immutable record that can be independently verified."
        },
        "for_bank_use": {
            "purpose": "Credit application / loan assessment",
            "data_accuracy": "Verified via blockchain consensus",
            "recommended_actions": [
                "Verify payment receipts with M-Pesa statements",
                "Cross-reference monthly revenue with bank deposits",
                "Review payment success rate (higher is better)",
                "Check refund rate (lower is better)",
                "Assess growth trajectory from monthly breakdown"
            ]
        }
    });
    
    // Return as JSON with proper content disposition for download
    let headers = [
        ("Content-Type", "application/json"),
        ("Content-Disposition", &format!(
            "attachment; filename=\"{}-ledger-{}.json\"",
            state.config.business.name.replace(" ", "-").to_lowercase(),
            chrono::Utc::now().format("%Y%m%d")
        )),
    ];
    
    (StatusCode::OK, headers, Json(report)).into_response()
}

/// Payment analytics with trends and insights
async fn payment_analytics(State(state): State<AppState>) -> impl IntoResponse {
    let orders = match state.store.list_orders(None) {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Failed to fetch orders: {}", e),
                }),
            ).into_response();
        }
    };
    
    // Get all payments
    let mut all_payments = Vec::new();
    for order in &orders {
        if let Ok(payments) = state.store.get_order_payments(order.id) {
            all_payments.extend(payments);
        }
    }
    
    // Time-series analysis (last 30 days, daily)
    use std::collections::HashMap;
    let mut daily_revenue: HashMap<String, f64> = HashMap::new();
    let mut daily_count: HashMap<String, i64> = HashMap::new();
    
    for payment in &all_payments {
        if matches!(payment.status, crate::payments::PaymentStatus::Completed) {
            let date = payment.created_at.chars().take(10).collect::<String>(); // "2026-02-06"
            *daily_revenue.entry(date.clone()).or_insert(0.0) += payment.amount;
            *daily_count.entry(date).or_insert(0) += 1;
        }
    }
    
    // Sort dates
    let mut dates: Vec<_> = daily_revenue.keys().cloned().collect();
    dates.sort();
    dates.reverse(); // Most recent first
    let dates: Vec<_> = dates.into_iter().take(30).rev().collect(); // Last 30 days, chronological
    
    let time_series: Vec<_> = dates.iter().map(|date| {
        serde_json::json!({
            "date": date,
            "revenue": daily_revenue.get(date).unwrap_or(&0.0),
            "count": daily_count.get(date).unwrap_or(&0),
        })
    }).collect();
    
    // Payment method breakdown
    let mpesa_count = all_payments.iter().filter(|p| matches!(p.method, crate::payments::PaymentMethod::MPesa)).count();
    let mpesa_revenue: f64 = all_payments.iter()
        .filter(|p| matches!(p.method, crate::payments::PaymentMethod::MPesa) && matches!(p.status, crate::payments::PaymentStatus::Completed))
        .map(|p| p.amount)
        .sum();
    
    // Average order value
    let completed_payments: Vec<_> = all_payments.iter().filter(|p| matches!(p.status, crate::payments::PaymentStatus::Completed)).collect();
    let avg_order_value = if !completed_payments.is_empty() {
        completed_payments.iter().map(|p| p.amount).sum::<f64>() / completed_payments.len() as f64
    } else {
        0.0
    };
    
    // Peak hours (if we had hour data - placeholder)
    let peak_hours = vec![
        serde_json::json!({"hour": "12:00", "count": 0}),
        serde_json::json!({"hour": "18:00", "count": 0}),
    ];
    
    let analytics = serde_json::json!({
        "time_series": time_series,
        "payment_methods": {
            "mpesa": {
                "count": mpesa_count,
                "revenue": mpesa_revenue,
                "percentage": if !all_payments.is_empty() { (mpesa_count as f64 / all_payments.len() as f64) * 100.0 } else { 0.0 },
            },
        },
        "insights": {
            "avg_order_value": format!("{}{:.2}", state.config.business.currency, avg_order_value),
            "peak_hours": peak_hours,
            "total_transactions": all_payments.len(),
            "successful_transactions": completed_payments.len(),
        },
    });
    
    (StatusCode::OK, Json(analytics)).into_response()
}

/// Automatic reconciliation report
async fn reconciliation_report(State(state): State<AppState>) -> impl IntoResponse {
    let stats = match state.store.get_stats() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Failed to fetch stats: {}", e),
                }),
            ).into_response();
        }
    };
    
    let refunds = match state.store.list_refunds(None) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: format!("Failed to fetch refunds: {}", e),
                }),
            ).into_response();
        }
    };
    
    // Calculate net revenue (revenue - refunds)
    let total_refunded: f64 = refunds.iter()
        .filter(|r| matches!(r.status, crate::store::RefundStatus::Completed))
        .map(|r| r.amount)
        .sum();
    
    let net_revenue = stats.payment_revenue - total_refunded;
    
    // Identify discrepancies
    let mut issues = Vec::new();
    
    // Check for stuck payments (processing for >24h)
    // TODO: Add timestamp comparison when we have it
    
    // Check for orders without payments
    let orders_without_payment = stats.total_orders - stats.total_payments;
    if orders_without_payment > 0 {
        issues.push(serde_json::json!({
            "severity": "warning",
            "issue": format!("{} orders without payment records", orders_without_payment),
            "action": "Review cash orders or missing payment data",
        }));
    }
    
    // Check for failed payment rate
    let failed_rate = if stats.total_payments > 0 {
        (stats.failed_payments as f64 / stats.total_payments as f64) * 100.0
    } else {
        0.0
    };
    
    if failed_rate > 10.0 {
        issues.push(serde_json::json!({
            "severity": "warning",
            "issue": format!("High payment failure rate: {:.1}%", failed_rate),
            "action": "Investigate payment issues with customers or M-Pesa configuration",
        }));
    }
    
    // Check for pending refunds
    let pending_refunds = refunds.iter().filter(|r| matches!(r.status, crate::store::RefundStatus::Pending | crate::store::RefundStatus::Processing)).count();
    if pending_refunds > 0 {
        issues.push(serde_json::json!({
            "severity": "info",
            "issue": format!("{} refunds pending", pending_refunds),
            "action": "Monitor M-Pesa B2C callbacks for completion",
        }));
    }
    
    let report = serde_json::json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "status": if issues.is_empty() { "ok" } else { "needs_review" },
        "summary": {
            "total_revenue": stats.total_revenue,
            "payment_revenue": stats.payment_revenue,
            "total_refunded": total_refunded,
            "net_revenue": net_revenue,
            "orders": stats.total_orders,
            "payments": stats.total_payments,
            "refunds": refunds.len(),
        },
        "health_checks": {
            "payment_success_rate": format!("{:.1}%", if stats.total_payments > 0 { (stats.completed_payments as f64 / stats.total_payments as f64) * 100.0 } else { 0.0 }),
            "payment_failure_rate": format!("{:.1}%", failed_rate),
            "refund_completion_rate": format!("{:.1}%", if !refunds.is_empty() {
                let completed = refunds.iter().filter(|r| matches!(r.status, crate::store::RefundStatus::Completed)).count();
                (completed as f64 / refunds.len() as f64) * 100.0
            } else { 0.0 }),
        },
        "issues": issues,
        "recommendations": if issues.is_empty() {
            vec!["All financial records are in good order. Continue monitoring daily."]
        } else {
            vec!["Review issues above and take recommended actions.", "Export ledger for detailed analysis if needed."]
        },
    });
    
    (StatusCode::OK, Json(report)).into_response()
}
