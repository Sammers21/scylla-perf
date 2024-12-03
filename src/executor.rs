use std::sync::Arc;
use crate::reporter;

use anyhow::Result;
use rand::distributions::{Alphanumeric, DistString};
use rand::{random, Rng};
use scylla::transport::session::{CurrentDeserializationApi, GenericSession};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;

pub struct Executor {
    total_keys: usize,
    concurrency: usize,
    key_string_length: usize,
    value_blob_size: usize,
    reads_percentage: f32,
    writes_percentage: f32,

    key_values_range: Vec<KeyValue>,
}

pub struct KeyValue(String, Vec<u8>);

impl Clone for KeyValue {
    fn clone(&self) -> Self {
        KeyValue(self.0.clone(), self.1.clone())
    }
}

impl Executor {
    pub fn new(
        concurrency: usize,
        key_string_length: usize,
        value_blob_size: usize,
        reads_percentage: f32,
        total_keys: usize,
    ) -> Executor {
        if reads_percentage < 0.0 || reads_percentage > 1.0 {
            panic!("Reads percentage must be between 0.0 and 1.0");
        }
        Executor {
            total_keys,
            concurrency,
            key_string_length,
            value_blob_size,
            reads_percentage,
            writes_percentage: 1.0 - reads_percentage,
            key_values_range: generate_key_values_range(
                total_keys,
                key_string_length,
                value_blob_size,
            ),
        }
    }

    pub async fn start(
        &self,
        session: Arc<GenericSession<CurrentDeserializationApi>>,
    ) -> Result<(Sender<()>, tokio::task::JoinHandle<()>)> {
        println!("Starting executor...");
        let create_keyspace = "CREATE KEYSPACE IF NOT EXISTS test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }";
        let create_table =
            "CREATE TABLE IF NOT EXISTS test.test (key text PRIMARY KEY, value blob);";
        let write = "INSERT INTO test.test (key, value) VALUES (?, ?)";
        let read = "SELECT * FROM test.test WHERE key = ?";
        session.query_unpaged(create_keyspace, &[]).await?;
        session.query_unpaged(create_table, &[]).await?;
        let (stop_sender, mut stop_receiver): (Sender<()>, Receiver<()>) = oneshot::channel();
        let (queries_sender, mut queries_receiver): (mpsc::Sender<Duration>, mpsc::Receiver<Duration>) =
            mpsc::channel(1000);
        let queries_sender = Arc::new(queries_sender);
        println!("Inserting initial key-value pairs...");
        // fill the channel with initial queries
        for kv in &self.key_values_range {
            perform_write(session.clone(), kv.clone()).await?;
        }
        println!("Done inserting initial key-value pairs");
        println!("Starting queries...");
        let concurrency = self.concurrency.clone();
        let key_values_range = self.key_values_range.clone();
        let reads_percentage = self.reads_percentage.clone();
        let future = async move {
            let mut current_concurrency = 0;
            let mut reporter = reporter::Reporter::new();
            loop {
                let stop = stop_receiver.try_recv();
                // if stop signal received, stop executor
                if stop.is_ok() {
                    let res = session.query_unpaged("DROP KEYSPACE test", &[]).await;
                    if res.is_err() {
                        panic!("Error dropping keyspace: {:?}", res.err());
                    }
                    println!("Stopping executor...");
                    break;
                }
                let mut duration = queries_receiver.try_recv();
                while duration.is_ok() {
                    reporter.report_results(duration.unwrap());
                    current_concurrency -= 1;
                    duration = queries_receiver.try_recv();
                }
                // if there are still queries to be sent
                while current_concurrency < concurrency {
                    current_concurrency += 1;
                    let scp = session.clone();
                    let queries_sender_cpy = queries_sender.clone();
                    let kvs = key_values_range.clone();
                    let reads_percentage = reads_percentage.clone();
                    tokio::spawn(async move {
                        let kv = kvs.get(rand::thread_rng().gen_range(0..kvs.len())).unwrap();
                        let rng = random::<f32>();
                        let res = if rng < reads_percentage {
                            perform_read(scp, kv.clone()).await
                        } else {
                            perform_write(scp, kv.clone()).await
                        };
                        if res.is_err() {
                            panic!("Error executing query: {:?}", res.err());
                        }
                        queries_sender_cpy.send(res.unwrap()).await.unwrap();
                    });
                }
            }
            ()
        };
        let executor_thread = tokio::spawn(future);
        Ok((stop_sender, executor_thread))
    }
}

async fn perform_read(
    session: Arc<GenericSession<CurrentDeserializationApi>>,
    kv: KeyValue,
) -> Result<Duration> {
    let start = tokio::time::Instant::now();
    let read = "SELECT * FROM test.test WHERE key = ?";
    let res = session.query_unpaged(read, (kv.0.clone(),)).await;
    if res.is_err() {
        Err(anyhow::anyhow!("Row not found"))
    } else {
        let elapsed = start.elapsed();
        Ok(elapsed)
    }
}
async fn perform_write(
    session: Arc<GenericSession<CurrentDeserializationApi>>,
    kv: KeyValue,
) -> Result<Duration> {
    let start = tokio::time::Instant::now();
    let write = "INSERT INTO test.test (key, value) VALUES (?, ?)";
    let str: String = kv.0.clone();
    let vec: &Vec<u8> = &kv.1;
    session.query_unpaged(write, (str, vec)).await?;
    let elapsed = start.elapsed();
    Ok(elapsed)
}

fn generate_key_values_range(
    total_keys: usize,
    key_string_length: usize,
    value_blob_size: usize,
) -> Vec<KeyValue> {
    let string = format!(
        "Generating {total_keys} key-value pairs, key length: {key_string_length}, value size: {value_blob_size}"
    );
    println!("{}", string);
    let mut key_values_range = Vec::new();
    let rng = &mut rand::thread_rng();
    for i in 0..total_keys {
        // generate random key string of length key_string_length
        let key = Alphanumeric.sample_string(rng, key_string_length);
        // generate random value blob of size value_blob_size
        let mut value = Vec::new();
        for _ in 0..value_blob_size {
            value.push(rng.gen::<u8>());
        }
        key_values_range.push(KeyValue(key, value));
    }
    key_values_range
}
