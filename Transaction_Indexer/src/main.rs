use sled::Db;
use std::sync::mpsc;
use std::thread;

pub struct SledStorage {
    db: Db,
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
        self.db.insert(key, block_data);
        self.db.flush()?;
        Ok(())
    }

    fn get_block(&self, block_number: u64) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let key = block_number.to_be_bytes();
        let result = self.db.get(key);

        Ok(result.map(|ivec| ivec.to_vec()))
    }
}

fn poll_ethblocknumber(sender: mpsc::Sender<String>) {}

fn processor(receiver: mpsc::Receiver<String>) {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    let listener_handle = thread::spawn(move || poll_ethblocknumber(tx));

    let processor_handle = thread::spawn(move || processor(rx));

    listener_handle.join().unwrap();
    processor_handle.join().unwrap();

    Ok(())
}
