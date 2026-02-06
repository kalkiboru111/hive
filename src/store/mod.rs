//! SQLite storage layer.
//!
//! Manages persistent state for orders, vouchers, menu items, and conversation
//! state. Uses rusqlite with a simple synchronous API (wrapped in `Arc` for sharing).

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Thread-safe handle to the SQLite database.
#[derive(Clone)]
pub struct Store {
    conn: Arc<Mutex<Connection>>,
}

/// Stored order record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRecord {
    pub id: i64,
    pub customer_phone: String,
    pub items_json: String,
    pub subtotal: f64,
    pub delivery_fee: f64,
    pub total: f64,
    pub status: OrderStatus,
    pub location: Option<String>,
    pub voucher_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Order lifecycle states.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Preparing,
    Delivering,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Confirmed => "confirmed",
            Self::Preparing => "preparing",
            Self::Delivering => "delivering",
            Self::Delivered => "delivered",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "confirmed" => Self::Confirmed,
            "preparing" => Self::Preparing,
            "delivering" => Self::Delivering,
            "delivered" => Self::Delivered,
            "cancelled" => Self::Cancelled,
            _ => Self::Pending,
        }
    }
}

/// Stored voucher record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoucherRecord {
    pub id: i64,
    pub code: String,
    pub amount: f64,
    pub redeemed_by: Option<String>,
    pub created_at: String,
    pub redeemed_at: Option<String>,
}

/// Stats summary for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total_orders: i64,
    pub pending_orders: i64,
    pub delivered_orders: i64,
    pub total_revenue: f64,
    pub total_vouchers: i64,
    pub redeemed_vouchers: i64,
}

impl Store {
    /// Open (or create) the SQLite database and run migrations.
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open database at {}", db_path))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Run schema migrations
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS orders (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_phone  TEXT NOT NULL,
                items_json      TEXT NOT NULL,
                subtotal        REAL NOT NULL DEFAULT 0,
                delivery_fee    REAL NOT NULL DEFAULT 0,
                total           REAL NOT NULL,
                status          TEXT NOT NULL DEFAULT 'pending',
                location        TEXT,
                voucher_code    TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS vouchers (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                code        TEXT NOT NULL UNIQUE,
                amount      REAL NOT NULL,
                redeemed_by TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                redeemed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS conversations (
                phone       TEXT PRIMARY KEY,
                state_json  TEXT NOT NULL DEFAULT '\"Idle\"',
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS payments (
                id              TEXT PRIMARY KEY,
                order_id        INTEGER NOT NULL,
                amount          REAL NOT NULL,
                currency        TEXT NOT NULL DEFAULT 'KES',
                method          TEXT NOT NULL,
                status          TEXT NOT NULL DEFAULT 'pending',
                phone           TEXT NOT NULL,
                reference       TEXT NOT NULL,
                provider_ref    TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (order_id) REFERENCES orders(id)
            );

            CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
            CREATE INDEX IF NOT EXISTS idx_orders_phone ON orders(customer_phone);
            CREATE INDEX IF NOT EXISTS idx_vouchers_code ON vouchers(code);
            CREATE INDEX IF NOT EXISTS idx_payments_order ON payments(order_id);
            CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);
            ",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // ─── Orders ──────────────────────────────────────────────────────

    /// Insert a new order. Returns the order ID.
    pub fn create_order(
        &self,
        customer_phone: &str,
        items_json: &str,
        subtotal: f64,
        delivery_fee: f64,
        total: f64,
        voucher_code: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO orders (customer_phone, items_json, subtotal, delivery_fee, total, voucher_code)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![customer_phone, items_json, subtotal, delivery_fee, total, voucher_code],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Update order status.
    pub fn update_order_status(&self, order_id: i64, status: &OrderStatus) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE orders SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![status.as_str(), order_id],
        )?;
        Ok(())
    }

    /// Set the delivery location for an order.
    pub fn set_order_location(&self, order_id: i64, location: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE orders SET location = ?1, status = 'confirmed', updated_at = datetime('now') WHERE id = ?2",
            params![location, order_id],
        )?;
        Ok(())
    }

    /// Get a single order by ID.
    pub fn get_order(&self, order_id: i64) -> Result<Option<OrderRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, customer_phone, items_json, subtotal, delivery_fee, total, status, location, voucher_code, created_at, updated_at
             FROM orders WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![order_id], |row| {
            Ok(OrderRecord {
                id: row.get(0)?,
                customer_phone: row.get(1)?,
                items_json: row.get(2)?,
                subtotal: row.get(3)?,
                delivery_fee: row.get(4)?,
                total: row.get(5)?,
                status: OrderStatus::from_str(&row.get::<_, String>(6)?),
                location: row.get(7)?,
                voucher_code: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;
        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// List orders, optionally filtered by status.
    pub fn list_orders(&self, status_filter: Option<&OrderStatus>) -> Result<Vec<OrderRecord>> {
        let conn = self.conn.lock().unwrap();
        let (sql, param_values): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match status_filter {
            Some(status) => (
                "SELECT id, customer_phone, items_json, subtotal, delivery_fee, total, status, location, voucher_code, created_at, updated_at
                 FROM orders WHERE status = ?1 ORDER BY created_at DESC",
                vec![Box::new(status.as_str().to_string())],
            ),
            None => (
                "SELECT id, customer_phone, items_json, subtotal, delivery_fee, total, status, location, voucher_code, created_at, updated_at
                 FROM orders ORDER BY created_at DESC",
                vec![],
            ),
        };

        let mut stmt = conn.prepare(sql)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(OrderRecord {
                id: row.get(0)?,
                customer_phone: row.get(1)?,
                items_json: row.get(2)?,
                subtotal: row.get(3)?,
                delivery_fee: row.get(4)?,
                total: row.get(5)?,
                status: OrderStatus::from_str(&row.get::<_, String>(6)?),
                location: row.get(7)?,
                voucher_code: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Get recent orders for a customer.
    pub fn get_customer_orders(&self, phone: &str, limit: usize) -> Result<Vec<OrderRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, customer_phone, items_json, subtotal, delivery_fee, total, status, location, voucher_code, created_at, updated_at
             FROM orders WHERE customer_phone = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![phone, limit as i64], |row| {
            Ok(OrderRecord {
                id: row.get(0)?,
                customer_phone: row.get(1)?,
                items_json: row.get(2)?,
                subtotal: row.get(3)?,
                delivery_fee: row.get(4)?,
                total: row.get(5)?,
                status: OrderStatus::from_str(&row.get::<_, String>(6)?),
                location: row.get(7)?,
                voucher_code: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ─── Vouchers ────────────────────────────────────────────────────

    /// Create a new voucher. Returns the voucher ID.
    pub fn create_voucher(&self, code: &str, amount: f64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO vouchers (code, amount) VALUES (?1, ?2)",
            params![code, amount],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Look up a voucher by code.
    pub fn get_voucher(&self, code: &str) -> Result<Option<VoucherRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, code, amount, redeemed_by, created_at, redeemed_at
             FROM vouchers WHERE code = ?1",
        )?;
        let mut rows = stmt.query_map(params![code], |row| {
            Ok(VoucherRecord {
                id: row.get(0)?,
                code: row.get(1)?,
                amount: row.get(2)?,
                redeemed_by: row.get(3)?,
                created_at: row.get(4)?,
                redeemed_at: row.get(5)?,
            })
        })?;
        match rows.next() {
            Some(Ok(record)) => Ok(Some(record)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Redeem a voucher. Returns the voucher amount if successful.
    pub fn redeem_voucher(&self, code: &str, redeemed_by: &str) -> Result<Option<f64>> {
        let conn = self.conn.lock().unwrap();

        // Check if the voucher exists and hasn't been redeemed
        let mut stmt = conn.prepare(
            "SELECT amount FROM vouchers WHERE code = ?1 AND redeemed_by IS NULL",
        )?;
        let amount: Option<f64> = stmt
            .query_map(params![code], |row| row.get(0))?
            .next()
            .and_then(|r| r.ok());

        if let Some(amount) = amount {
            conn.execute(
                "UPDATE vouchers SET redeemed_by = ?1, redeemed_at = datetime('now') WHERE code = ?2",
                params![redeemed_by, code],
            )?;
            Ok(Some(amount))
        } else {
            Ok(None)
        }
    }

    /// List all vouchers.
    pub fn list_vouchers(&self) -> Result<Vec<VoucherRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, code, amount, redeemed_by, created_at, redeemed_at
             FROM vouchers ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(VoucherRecord {
                id: row.get(0)?,
                code: row.get(1)?,
                amount: row.get(2)?,
                redeemed_by: row.get(3)?,
                created_at: row.get(4)?,
                redeemed_at: row.get(5)?,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ─── Conversation State ──────────────────────────────────────────

    /// Get the conversation state JSON for a phone number.
    pub fn get_conversation_state(&self, phone: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT state_json FROM conversations WHERE phone = ?1",
        )?;
        let mut rows = stmt.query_map(params![phone], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(json)) => Ok(Some(json)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Save conversation state.
    pub fn save_conversation_state(&self, phone: &str, state_json: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO conversations (phone, state_json, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(phone) DO UPDATE SET state_json = ?2, updated_at = datetime('now')",
            params![phone, state_json],
        )?;
        Ok(())
    }

    // ─── Stats ───────────────────────────────────────────────────────

    /// Get aggregate stats for the dashboard.
    pub fn get_stats(&self) -> Result<Stats> {
        let conn = self.conn.lock().unwrap();

        let total_orders: i64 = conn.query_row(
            "SELECT COUNT(*) FROM orders",
            [],
            |row| row.get(0),
        )?;

        let pending_orders: i64 = conn.query_row(
            "SELECT COUNT(*) FROM orders WHERE status IN ('pending', 'confirmed', 'preparing', 'delivering')",
            [],
            |row| row.get(0),
        )?;

        let delivered_orders: i64 = conn.query_row(
            "SELECT COUNT(*) FROM orders WHERE status = 'delivered'",
            [],
            |row| row.get(0),
        )?;

        let total_revenue: f64 = conn.query_row(
            "SELECT COALESCE(SUM(total), 0) FROM orders WHERE status = 'delivered'",
            [],
            |row| row.get(0),
        )?;

        let total_vouchers: i64 = conn.query_row(
            "SELECT COUNT(*) FROM vouchers",
            [],
            |row| row.get(0),
        )?;

        let redeemed_vouchers: i64 = conn.query_row(
            "SELECT COUNT(*) FROM vouchers WHERE redeemed_by IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        Ok(Stats {
            total_orders,
            pending_orders,
            delivered_orders,
            total_revenue,
            total_vouchers,
            redeemed_vouchers,
        })
    }

    // ─── Payments ────────────────────────────────────────────────────

    /// Create a new payment record. Returns the payment ID.
    pub fn create_payment(
        &self,
        payment_id: &str,
        order_id: i64,
        amount: f64,
        currency: &str,
        method: &str,
        phone: &str,
        reference: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO payments (id, order_id, amount, currency, method, phone, reference)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![payment_id, order_id, amount, currency, method, phone, reference],
        )?;
        Ok(())
    }

    /// Update payment status and provider reference.
    pub fn update_payment_status(
        &self,
        payment_id: &str,
        status: &str,
        provider_ref: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE payments SET status = ?1, provider_ref = ?2, updated_at = datetime('now') WHERE id = ?3",
            params![status, provider_ref, payment_id],
        )?;
        Ok(())
    }

    /// Get payment by ID.
    pub fn get_payment(&self, payment_id: &str) -> Result<Option<crate::payments::Payment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, order_id, amount, currency, method, status, phone, reference, provider_ref, created_at, updated_at
             FROM payments WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![payment_id], |row| {
                Ok(crate::payments::Payment {
                    id: row.get(0)?,
                    order_id: row.get(1)?,
                    amount: row.get(2)?,
                    currency: row.get(3)?,
                    method: serde_json::from_str(&format!(r#""{}""#, row.get::<_, String>(4)?)).unwrap(),
                    status: match row.get::<_, String>(5)?.as_str() {
                        "pending" => crate::payments::PaymentStatus::Pending,
                        "processing" => crate::payments::PaymentStatus::Processing,
                        "completed" => crate::payments::PaymentStatus::Completed,
                        "failed" => crate::payments::PaymentStatus::Failed,
                        "cancelled" => crate::payments::PaymentStatus::Cancelled,
                        _ => crate::payments::PaymentStatus::Pending,
                    },
                    phone: row.get(6)?,
                    reference: row.get(7)?,
                    provider_ref: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            });

        match result {
            Ok(payment) => Ok(Some(payment)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get payments for an order.
    pub fn get_order_payments(&self, order_id: i64) -> Result<Vec<crate::payments::Payment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, order_id, amount, currency, method, status, phone, reference, provider_ref, created_at, updated_at
             FROM payments WHERE order_id = ?1 ORDER BY created_at DESC",
        )?;

        let payments = stmt
            .query_map(params![order_id], |row| {
                Ok(crate::payments::Payment {
                    id: row.get(0)?,
                    order_id: row.get(1)?,
                    amount: row.get(2)?,
                    currency: row.get(3)?,
                    method: serde_json::from_str(&format!(r#""{}""#, row.get::<_, String>(4)?)).unwrap(),
                    status: match row.get::<_, String>(5)?.as_str() {
                        "pending" => crate::payments::PaymentStatus::Pending,
                        "processing" => crate::payments::PaymentStatus::Processing,
                        "completed" => crate::payments::PaymentStatus::Completed,
                        "failed" => crate::payments::PaymentStatus::Failed,
                        "cancelled" => crate::payments::PaymentStatus::Cancelled,
                        _ => crate::payments::PaymentStatus::Pending,
                    },
                    phone: row.get(6)?,
                    reference: row.get(7)?,
                    provider_ref: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(payments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> Store {
        Store::new(":memory:").expect("Failed to create in-memory store")
    }

    #[test]
    fn test_create_and_get_order() {
        let store = test_store();
        let id = store
            .create_order("+27123456789", r#"[{"name":"Kota","price":35}]"#, 35.0, 10.0, 45.0, None)
            .unwrap();
        let order = store.get_order(id).unwrap().unwrap();
        assert_eq!(order.customer_phone, "+27123456789");
        assert_eq!(order.total, 45.0);
        assert_eq!(order.status, OrderStatus::Pending);
    }

    #[test]
    fn test_voucher_lifecycle() {
        let store = test_store();
        store.create_voucher("TEST123", 50.0).unwrap();

        let voucher = store.get_voucher("TEST123").unwrap().unwrap();
        assert_eq!(voucher.amount, 50.0);
        assert!(voucher.redeemed_by.is_none());

        let amount = store.redeem_voucher("TEST123", "+27123456789").unwrap();
        assert_eq!(amount, Some(50.0));

        // Can't redeem twice
        let again = store.redeem_voucher("TEST123", "+27999999999").unwrap();
        assert_eq!(again, None);
    }

    #[test]
    fn test_conversation_state() {
        let store = test_store();
        let phone = "+27123456789";

        assert!(store.get_conversation_state(phone).unwrap().is_none());

        store.save_conversation_state(phone, r#""ViewingMenu""#).unwrap();
        let state = store.get_conversation_state(phone).unwrap().unwrap();
        assert_eq!(state, r#""ViewingMenu""#);

        // Upsert works
        store.save_conversation_state(phone, r#""Idle""#).unwrap();
        let state = store.get_conversation_state(phone).unwrap().unwrap();
        assert_eq!(state, r#""Idle""#);
    }
}
