mod executor;
mod reporter;

use anyhow::Result;
use scylla::transport::session::{CurrentDeserializationApi, GenericSession};
use scylla::SessionBuilder;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let concurrency = 100;
    let duration = Duration::from_secs(10);
    let scyllaHosts = "127.0.0.1:9042";
    let hostsSplit = scyllaHosts.split(",");
    let mut builder = SessionBuilder::new();
    for host in hostsSplit {
        builder = builder.known_node(host);
    }
    let session: Arc<GenericSession<CurrentDeserializationApi>> = Arc::new(builder.build().await?);
    let mut executor = executor::Executor::new(concurrency, 32, 4 * 1024, 0.5, 1000);
    let (stop_sender, executor_thread) = executor.start(session).await?;
    tokio::time::sleep(duration).await;
    if let Err(e) = stop_sender.send(()) {
        println!("Error sending stop signal: {:?}", e);
    }
    executor_thread.await?;
    Ok(())
}
