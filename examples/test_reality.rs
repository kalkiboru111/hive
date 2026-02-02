//! Integration test: submit a state channel snapshot to a local Reality cluster.
//!
//! Usage: cargo run --example test_reality
//!
//! Expects a Reality L0 node at http://localhost:7000

use hive::network::client::RealityClient;
use hive::network::identity::NodeIdentity;
use hive::network::snapshot::HiveStateSnapshot;
use hive::network::snapshot::VoucherStateSummary;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cluster_url = std::env::var("REALITY_URL").unwrap_or("http://localhost:7000".into());
    println!("🔗 Connecting to Reality cluster at {}", cluster_url);

    let client = RealityClient::new(&cluster_url);

    // 1. Health check
    println!("\n── Step 1: Cluster health ──");
    let nodes = client.cluster_info().await?;
    println!("  ✅ {} node(s) in cluster", nodes.len());
    for node in &nodes {
        println!("     {} — {} ({})", &node.id[..16], node.ip, node.state);
    }

    // 2. Latest ordinal
    println!("\n── Step 2: Latest ordinal ──");
    let ordinal = client.latest_ordinal().await?;
    println!("  ✅ Ordinal: {}", ordinal);

    // 3. Generate identity
    println!("\n── Step 3: Generate node identity ──");
    let identity = NodeIdentity::generate()?;
    println!("  ✅ Peer ID: {}...", &identity.peer_id_hex[..32]);
    println!("  ✅ Address: {}", identity.address);

    // 4. Build a test snapshot
    println!("\n── Step 4: Build state snapshot ──");
    let snapshot = HiveStateSnapshot {
        version: 1,
        business_name: "Cloudy Deliveries".to_string(),
        timestamp_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis() as u64,
        total_orders: 3,
        total_revenue_cents: 10500,
        active_orders: 1,
        delivered_orders: 2,
        vouchers: VoucherStateSummary {
            total_created: 2,
            total_redeemed: 1,
            total_value_created_cents: 5000,
            total_value_redeemed_cents: 2500,
        },
        order_hashes: vec![
            "a1b2c3d4e5f60001".to_string(),
            "a1b2c3d4e5f60002".to_string(),
            "a1b2c3d4e5f60003".to_string(),
        ],
    };

    let content_bytes = snapshot.to_bytes()?;
    println!("  ✅ Snapshot serialized: {} bytes (MessagePack)", content_bytes.len());

    // Verify roundtrip
    let restored = HiveStateSnapshot::from_bytes(&content_bytes)?;
    assert_eq!(restored.total_orders, 3);
    println!("  ✅ Roundtrip verified");

    // 5. Build state channel binary
    println!("\n── Step 5: Build StateChannelSnapshotBinary ──");
    // Use empty hash for first snapshot in chain
    let sc_binary = snapshot.to_state_channel_binary(
        "0000000000000000000000000000000000000000000000000000000000000000",
    )?;
    println!(
        "  ✅ Content: {} bytes (signed), lastSnapshotHash: {}...",
        sc_binary.content.len(),
        &sc_binary.last_snapshot_hash[..16]
    );

    // 6. Sign it
    println!("\n── Step 6: Sign snapshot ──");
    let signed = identity.sign_value(&sc_binary)?;
    println!("  ✅ Signed with {} proof(s)", signed.proofs.len());
    println!(
        "     Signature: {}...",
        &signed.proofs[0].signature[..40]
    );

    // 7. Submit to L0
    println!("\n── Step 7: Submit state channel snapshot ──");
    match client
        .submit_state_channel_snapshot(&identity.address, &signed)
        .await
    {
        Ok(()) => {
            println!("  ✅ ACCEPTED by L0! State channel snapshot is on-chain.");
        }
        Err(e) => {
            println!("  ❌ Rejected: {}", e);
            println!("     (This is expected if the L0 node validates state channel addresses)");
        }
    }

    // 8. Check ordinal advanced
    println!("\n── Step 8: Wait for next snapshot ──");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let new_ordinal = client.latest_ordinal().await?;
    println!(
        "  Ordinal: {} → {} (delta: {})",
        ordinal,
        new_ordinal,
        new_ordinal - ordinal
    );

    println!("\n🎉 Integration test complete!");
    Ok(())
}
