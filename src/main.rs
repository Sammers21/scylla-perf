mod executor;
mod reporter;

use anyhow::Result;
use clap::Parser;
use parse_duration::parse;
use scylla::transport::session::{CurrentDeserializationApi, GenericSession, PoolSize};
use scylla::SessionBuilder;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "ScyllaDB performance tool",
    long_about = "A tool for benchmarking Scylla DB performance"
)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "100",
        help = "Number of concurrent requests at any given moment of time"
    )]
    concurrency: usize,

    #[arg(short, long, value_parser = parse, default_value = "10s", help = "Duration of the benchmark"
    )]
    duration: Duration,

    #[arg(
        short,
        long,
        default_value = "127.0.0.1:9042",
        help = "Comma-separated list of Scylla hosts with their ports. Example: '127.0.0.1:9042,127.0.0.2:9042,127.0.0.3:9042'"
    )]
    scylla_hosts: String,

    #[arg(
        short,
        long,
        default_value = "32",
        help = "Length of the key strings in the database"
    )]
    key_string_length: usize,

    #[arg(
        short,
        long,
        default_value = "1024",
        help = "Size of the value blobs in the database"
    )]
    value_blob_size: usize,

    #[arg(
        short,
        long,
        default_value = "0.5",
        help = "Percentage of reads in the workload. The rest will be writes. Must be between 0.0 and 1.0"
    )]
    reads_percentage: f32,

    #[arg(
        short,
        long,
        default_value = "1000",
        help = "Total number of keys in the database"
    )]
    total_keys: usize,

    #[arg(short = 'u', long, default_value = "cassandra", help = "Scylla user")]
    pub user: String,

    #[arg(long, default_value = "cassandra", help = "Scylla password")]
    pub password: String,

    #[arg(
        short,
        long,
        default_value = "10",
        help = "Number of connections per host in the pool"
    )]
    pub pool_size: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    let pass_first4 = args.password.chars().take(4).collect::<String>();
    let pass_after4 = args
        .password
        .chars()
        .skip(4)
        .map(|_| '*')
        .collect::<String>();
    let pass = pass_first4 + &pass_after4;
    println!(
        "Args: duration: {}s, scylla_host: {}, pool_size: {}, user: {}, password: {}, key_string_length: {}, value_blob_size: {}, reads_percentage: {}, total_keys: {}",
        args.duration.as_secs_f64(),
        args.scylla_hosts,
        args.pool_size,
        args.user,
        pass,
        args.key_string_length,
        args.value_blob_size,
        args.reads_percentage,
        args.total_keys,
    );
    let hosts_split = args.scylla_hosts.split(",");
    let mut builder = SessionBuilder::new()
        .user(&args.user, &args.password)
        .pool_size(PoolSize::PerHost(NonZeroUsize::new(args.pool_size).unwrap()))
        .keyspaces_to_fetch(["test"]);
    for host in hosts_split {
        builder = builder.known_node(host);
    }
    let session: Arc<GenericSession<CurrentDeserializationApi>> = Arc::new(builder.build().await?);
    let executor = executor::Executor::new(
        args.concurrency,
        args.key_string_length,
        args.value_blob_size,
        args.reads_percentage,
        args.total_keys,
    );
    let (stop_sender, executor_thread) = executor.start(session).await?;
    tokio::time::sleep(args.duration).await;
    if let Err(e) = stop_sender.send(()) {
        println!("Error sending stop signal: {:?}", e);
    }
    executor_thread.await?;
    Ok(())
}
