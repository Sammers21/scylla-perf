mod executor;
mod reporter;

use anyhow::Result;
use scylla::SessionBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let concurrency = 10;
    let duration = Duration::from_secs(10);
    let readsPercentage = 0.5;
    let writesPercentage = 0.5;
    let valueSizeBytes = 100;
    let keySizeBytes = 16;
    let scyllaHosts = "127.0.0.1:9042";
    let hostsSplit = scyllaHosts.split(",");
    let testKeyspace = "test";
    let mut builder = SessionBuilder::new();
    for host in hostsSplit {
        builder = builder.known_node(host);
    }
    let session = builder.build().await?;
    let reporter = reporter::Reporter::new();
    let executor = executor::Executor::new(
        session,
        concurrency,
        reporter,
    );
    executor.start();
    tokio::time::sleep(duration).await;
    executor.stop();
    Ok(())
}
