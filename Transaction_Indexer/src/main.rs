use sled::Db;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

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
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn get_block(&self, block_number: u64) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>>;
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key = block_number.to_be_bytes();
        self.db.insert(key, block_data)?;
        self.db.flush()?;
        Ok(())
    }

    fn get_block(&self, block_number: u64) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let key = block_number.to_be_bytes();
        let result = self.db.get(key)?;

        Ok(result.map(|ivec| ivec.to_vec()))
    }
}

async fn poll_ethblocknumber(_sender: Sender<Block>) {}

async fn processor(mut receiver: mpsc::Receiver<Block>) {
    let db = sled::open("my_db").unwrap();
    let storage = SledStorage::new(db);

    while let Some(block) = receiver.recv().await {
        let block_number = block.number;
        let block_data = block.data;

        for tx in block.transactions {
            let _tx_hash = tx.hash;
            let _tx_from = tx.from;
            let _tx_to = tx.to;
            let _tx_value = tx.value;

            // future: update balances here
        }

        storage.put_block(block_number, block_data).unwrap();
    }
}

async fn fetch_block(block_number: u64) -> Result<Block, Box<dyn std::error::Error>> {
    Ok(Block {
        number: block_number,
        data: vec![],
        transactions: vec![],
    })
}

async fn backfilling(
    sender: Sender<Block>,
    current_head: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    for block_number in 0..=current_head {
        let tx = sender.clone();

        tokio::spawn(async move {
            let block = fetch_block(block_number).await.unwrap();

            tx.send(block).await.unwrap();
        });
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    let listener_tx = tx.clone();
    let backfill_tx = tx.clone();

    let listener_handle = tokio::spawn(async move {
        poll_ethblocknumber(listener_tx).await;
    });

    let processor_handle = tokio::spawn(async move {
        processor(rx).await;
    });

    let current_head = 1000;

    let backfill_handle = tokio::spawn(async move {
        backfilling(backfill_tx, current_head).await.unwrap();
    });

    listener_handle.await.unwrap();
    processor_handle.await.unwrap();
    backfill_handle.await.unwrap();

    Ok(())
}
