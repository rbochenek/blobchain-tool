#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use anyhow::Result;
use clap::{Parser, Subcommand};
use subxt::client::OnlineClientT;
use subxt::tx::Signer;
use subxt::{Config, OnlineClient, SubstrateConfig};
use subxt_signer::sr25519::{dev, Keypair};

type AccountId = <SubstrateConfig as Config>::AccountId;

// Generate an interface from node metadata
#[subxt::subxt(runtime_metadata_path = "./artifacts/blobchain.scale")]
pub mod substrate {}

// Clap arguments
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Monitor blobchain for new blocks and events
    Monitor,
    /// View Blobmanager state
    Show,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::builder()
        // Default log level
        .filter(None, log::LevelFilter::Debug)
        .init();

    // Parse clap arguments
    let cli = Cli::parse();

    // Hard-coded account
    let uploader_keypair: Keypair = dev::bob();

    // Connect to node
    let client = OnlineClient::<SubstrateConfig>::new().await?;

    match &cli.command {
        Commands::Monitor => {
            let mut blocks_sub = client.blocks().subscribe_all().await?;

            while let Some(block) = blocks_sub.next().await {
                let block = block?;

                let block_number = block.header().number;
                let block_hash = block.hash();

                log::debug!("New block: {} hash: {:?}", block_number, block_hash);

                let events = block.events().await?;

                for ev_blobstored in events.find::<substrate::blob_manager::events::BlobStored>() {
                    let event = ev_blobstored?;
                    log::info!("Event BlobStored");
                }

                for event in events.iter() {
                    let event = event?;

                    let pallet = event.pallet_name();
                    let variant = event.variant_name();
                    let field_values = event.field_values()?;

                    log::debug!("Event {pallet}::{variant}: {field_values}");
                }
            }
        }
        Commands::Show => {
            // Uploader
            let uploader_query = substrate::storage().blob_manager().uploader();
            let result = client
                .storage()
                .at_latest()
                .await?
                .fetch(&uploader_query)
                .await?;
            if let Some(uploader) = result {
                log::info!("Uploader: {}", uploader.to_string());
            } else {
                log::warn!("Uploader not set");
            }

            // Blobs
            let blobs_query = substrate::storage().blob_manager().blobs_iter();
            let mut results = client
                .storage()
                .at_latest()
                .await?
                .iter(blobs_query)
                .await?;
            while let Some(Ok(kv)) = results.next().await {
                log::info!("Keys decoded: {:?}", kv.keys);
                log::info!("Key: 0x{}", hex::encode(&kv.key_bytes));
                log::info!("Value: {:?}", kv.value);
            }

            // StorageVersion
            let storage_version = client
                .storage()
                .at_latest()
                .await?
                .storage_version("BlobManager")
                .await?;
            log::info!("StorageVersion: {storage_version}");
        }
    }

    Ok(())
}

/// Get blobs for a given block number
fn get_blobs_for_blocknumber(block_number: u32) -> Result<Vec<Vec<u8>>> {
    let output = vec![];

    Ok(output)
}

/// Call extrinsic to store blob
async fn store_blob(acc: AccountId) -> Result<()> {
    // let tx_payload = substrate::tx().blob_manager().upload_blob

    Ok(())
}
