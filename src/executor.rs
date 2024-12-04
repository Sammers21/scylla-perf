use crate::reporter::{QueryType, Reporter};
use anyhow::Result;
use rand::distributions::{Alphanumeric, DistString};
use rand::{random, Rng};
use scylla::prepared_statement::PreparedStatement;
use scylla::transport::session::{CurrentDeserializationApi, GenericSession};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::sync::{broadcast, oneshot, Mutex};

pub struct Executor {
    concurrency: usize,
    reads_percentage: f32,
    reporter: Arc<Mutex<Box<dyn Reporter + Send>>>,
    key_values_range: Vec<KeyValue>,
    drop_test_keyspace: bool,
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
        reporter: Box<dyn Reporter + Send>,
        drop_test_keyspace: bool,
    ) -> Executor {
        if reads_percentage < 0.0 || reads_percentage > 1.0 {
            panic!("Reads percentage must be between 0.0 and 1.0");
        }
        Executor {
            concurrency,
            reads_percentage,
            key_values_range: generate_key_values_range(
                total_keys,
                key_string_length,
                value_blob_size,
            ),
            reporter: Arc::new(Mutex::new(reporter)),
            drop_test_keyspace,
        }
    }

    pub async fn start(
        &mut self,
        session: Arc<GenericSession<CurrentDeserializationApi>>,
    ) -> Result<(Sender<()>, tokio::task::JoinHandle<()>)> {
        println!("Starting executor...");
        let create_keyspace = "CREATE KEYSPACE IF NOT EXISTS test WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 }";
        let create_table =
            "CREATE TABLE IF NOT EXISTS test.test (key text PRIMARY KEY, value blob);";
        session.query_unpaged(create_keyspace, &[]).await?;
        session.query_unpaged(create_table, &[]).await?;
        let prepared_read = session
            .prepare("SELECT * FROM test.test WHERE key = ?")
            .await?;
        let prepared_write = session
            .prepare("INSERT INTO test.test (key, value) VALUES (?, ?)")
            .await?;
        let (tx_stop_coordinator, mut rx_stop_coordinator): (Sender<()>, Receiver<()>) =
            oneshot::channel();
        let (tx_stop_metrics_reporter, mut rx_stop_metrics_reporter): (Sender<()>, Receiver<()>) =
            oneshot::channel();
        let (tx_metrics, mut rx_metrics): (
            broadcast::Sender<(QueryType, Duration)>,
            broadcast::Receiver<(QueryType, Duration)>,
        ) = broadcast::channel(10000);
        println!("Inserting initial key-value pairs...");
        for kv in &self.key_values_range {
            perform_write(session.clone(), prepared_write.clone(), kv.clone()).await?;
        }
        println!("Done inserting initial key-value pairs");
        println!("Starting queries...");
        let concurrency = self.concurrency.clone();
        let key_values_range = self.key_values_range.clone();
        let reads_percentage = self.reads_percentage.clone();
        let reporter_clone = self.reporter.clone();
        let _ = tokio::task::spawn(async move {
            let mut reporter = reporter_clone.lock().await;
            loop {
                if rx_stop_metrics_reporter.try_recv().is_ok() {
                    println!("Stopping metrics reporter...");
                    break;
                }
                while let Ok(res) = rx_metrics.try_recv() {
                    (*reporter).report_results(res.0, res.1);
                }
            }
            ()
        });
        let drop_test_keyspace_clone = self.drop_test_keyspace.clone();
        let coordinator_thread = tokio::task::spawn(async move {
            let current_concurrency = Arc::new(AtomicUsize::new(0));
            loop {
                let stop = rx_stop_coordinator.try_recv();
                // if stop signal received, stop executor
                if stop.is_ok() {
                    println!("Coordinator received stop signal, waiting for concurrent tasks to finish...");
                    while current_concurrency.load(std::sync::atomic::Ordering::Relaxed) > 0 {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    println!("All tasks finished, stopping metrics reporter...");
                    tx_stop_metrics_reporter.send(()).unwrap();
                    if drop_test_keyspace_clone {
                        println!("Dropping test keyspace...");
                        let res = session.query_unpaged("DROP KEYSPACE test", &[]).await;
                        if res.is_err() {
                            panic!("Error dropping keyspace: {:?}", res.err());
                        }
                        println!("Dropped test keyspace");
                    }
                    break;
                }
                // if there are still queries to be sent
                while current_concurrency.load(std::sync::atomic::Ordering::Relaxed) < concurrency {
                    current_concurrency.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let session_clone = session.clone();
                    let tx_metrics_clone = tx_metrics.clone();
                    let kvs = key_values_range.clone();
                    let reads_percentage = reads_percentage.clone();
                    let pread = prepared_read.clone();
                    let pwrite = prepared_write.clone();
                    let current_concurrency_clone = Arc::clone(&current_concurrency);
                    tokio::spawn(async move {
                        let kv = kvs.get(rand::thread_rng().gen_range(0..kvs.len())).unwrap();
                        let rng = random::<f32>();
                        let res = if rng < reads_percentage {
                            (
                                QueryType::Read,
                                perform_read(session_clone, pread, kv.clone()).await,
                            )
                        } else {
                            (
                                QueryType::Write,
                                perform_write(session_clone, pwrite, kv.clone()).await,
                            )
                        };
                        if res.1.is_err() {
                            println!("Error executing query: {:?}, {:?}", res.0, res.1.err());
                        } else {
                            let q_type = res.0;
                            let elapsed = res.1.unwrap();
                            loop {
                                let result = tx_metrics_clone.send((q_type, elapsed));
                                if result.is_ok() {
                                    break;
                                } else {
                                    println!("Error sending metrics: {:?}", result.err());
                                    break;
                                }
                            }
                        }
                        current_concurrency_clone
                            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    });
                }
            }
        });
        Ok((tx_stop_coordinator, coordinator_thread))
    }
}

async fn perform_read(
    session: Arc<GenericSession<CurrentDeserializationApi>>,
    ps: PreparedStatement,
    kv: KeyValue,
) -> Result<Duration> {
    let start = tokio::time::Instant::now();
    let res = session.execute_unpaged(&ps, (kv.0.clone(),)).await;
    if res.is_err() {
        Err(anyhow::anyhow!("Row not found"))
    } else {
        let elapsed = start.elapsed();
        Ok(elapsed)
    }
}
async fn perform_write(
    session: Arc<GenericSession<CurrentDeserializationApi>>,
    ps: PreparedStatement,
    kv: KeyValue,
) -> Result<Duration> {
    let start = tokio::time::Instant::now();
    let str: String = kv.0.clone();
    let vec: &Vec<u8> = &kv.1;
    let _ = session.execute_unpaged(&ps, (str, vec)).await;
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
    for _ in 0..total_keys {
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
