use sled::Db;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

pub struct SledStorage {
    db: Db,
}
struct Transaction {
    hash: String,
    from: String,
    to: String,
    value: u64,
}

struct Block {
    number: u64,
    data: Vec<u8>,
    transactions: Vec<Transaction>,
}

pub trait Storage {
    fn put_block(
        &self,
        block_number: u64,
        block_data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn get_block(
        &self,
        block_number: u64,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>>;
}

impl SledStorage {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl Storage for SledStorage {
    fn put_block(
        &self,
        block_number: u64,
        block_data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        let key = block_number.to_be_bytes();
        self.db.insert(key, block_data)?;

        Ok(())
    }

    fn get_block(
        &self,
        block_number: u64,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {

        let key = block_number.to_be_bytes();
        let result = self.db.get(key)?;

        Ok(result.map(|ivec| ivec.to_vec()))
    }
}

async fn poll_ethblocknumber(
    _sender: Sender<Block>,
    shutdown: CancellationToken,
) {

    loop {
        select! {

            _ = shutdown.cancelled() => {
                println!("Listener shutting down");
                break;
            }

            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {

                // future RPC polling logic here
                println!("Polling new blocks...");
            }
        }
    }
}

async fn processor(
    mut receiver: mpsc::Receiver<Block>,
    shutdown: CancellationToken,
) {

    let db = sled::open("my_db").unwrap();
    let storage = SledStorage::new(db);

    loop {

        select! {

            maybe_block = receiver.recv() => {

                match maybe_block {

                    Some(block) => {

                        let block_number = block.number;
                        let block_data = block.data;

                        for tx in block.transactions {

                            let _tx_hash = tx.hash;
                            let _tx_from = tx.from;
                            let _tx_to = tx.to;
                            let _tx_value = tx.value;

                        }

                        storage.put_block(block_number, block_data).unwrap();
                    }

                    None => {
                        println!("Channel closed. Processor exiting.");
                        break;
                    }
                }
            }

            _ = shutdown.cancelled() => {

                println!("Processor received shutdown signal");

                while let Some(block) = receiver.recv().await {

                    storage.put_block(block.number, block.data).unwrap();

                }

                println!("Processor finished draining channel");
                break;
            }
        }
    }
}

async fn fetch_block(
    block_number: u64,
) -> Result<Block, Box<dyn std::error::Error + Send + Sync>> {

    Ok(Block {
        number: block_number,
        data: vec![],
        transactions: vec![],
    })
}

async fn backfilling(
    sender: Sender<Block>,
    current_head: u64,
    shutdown: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let max_parallel_tasks = 50;
    let semaphore = Arc::new(Semaphore::new(max_parallel_tasks));

    for block_number in 0..=current_head {

        if shutdown.is_cancelled() {
            println!("Backfill stopping due to shutdown");
            break;
        }

        let _permit = semaphore.clone().acquire_owned().await?;
        let tx = sender.clone();
        let shutdown_clone = shutdown.clone();

        tokio::spawn(async move {

            if shutdown_clone.is_cancelled() {
                return;
            }

            match fetch_block(block_number).await {

                Ok(block) => {

                    if tx.send(block).await.is_err() {
                        println!("Channel closed");
                    }

                }

                Err(e) => {
                    println!("Error fetching block {}: {}", block_number, e);
                }
            }
        });
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let (tx, rx) = mpsc::channel(100);

    let shutdown_token = CancellationToken::new();

    let listener_tx = tx.clone();
    let backfill_tx = tx.clone();

    let listener_shutdown = shutdown_token.clone();
    let processor_shutdown = shutdown_token.clone();
    let backfill_shutdown = shutdown_token.clone();

    let current_head = 1000;

    let listener_handle = tokio::spawn(async move {

        poll_ethblocknumber(listener_tx, listener_shutdown).await;

    });

    let processor_handle = tokio::spawn(async move {

        processor(rx, processor_shutdown).await;

    });

    let backfill_handle = tokio::spawn(async move {

        backfilling(backfill_tx, current_head, backfill_shutdown)
            .await
            .unwrap();

    });

    tokio::signal::ctrl_c().await?;

    println!("Ctrl+C received. Initiating shutdown.");

    shutdown_token.cancel();

    listener_handle.await?;
    backfill_handle.await?;
    processor_handle.await?;

    println!("Shutdown complete.");

    Ok(())
}