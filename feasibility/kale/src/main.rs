mod albedo;
mod contracts;
mod rpc;

use contracts::kale::Kale;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Galactic Playground - KALE Plant Transaction ===\n");

    // Testnet configuration
    const TESTNET_RPC: &str = "https://soroban-testnet.stellar.org";
    const TESTNET_CONTRACT: &str = "CDSWUUXGPWDZG76ISK6SUCVPZJMD5YUV66J2FXFXFGDX25XKZJIEITAO";
    const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";

    // Create KALE contract client
    println!("Connecting to KALE contract on testnet...");
    let kale = Kale::new(TESTNET_RPC, TESTNET_CONTRACT, TESTNET_PASSPHRASE)?;
    println!("✓ Connected to KALE contract: {}\n", TESTNET_CONTRACT);

    // Get current block index
    println!("Fetching current farm block...");
    let block_index = kale.get_block_index().await?;
    println!("✓ Current block index: {}\n", block_index);

    // Start the authentication and plant transaction flow
    println!("Starting authentication and plant transaction flow...");
    let (public_key, tx_hash) = albedo::authenticate_and_plant(kale).await?;

    println!("\n=== Transaction Complete ===");
    println!("Public key: {}", public_key);
    println!("Transaction hash: {}", tx_hash);
    println!("\nYou can view the transaction on Stellar Expert:");
    println!("https://stellar.expert/explorer/testnet/tx/{}", tx_hash);

    Ok(())
}
