mod executor;
mod reporter;

use crate::reporter::{PercentileReporter, Reporter};
use anyhow::Result;
use clap::Parser;
use parse_duration::parse;
use reporter::SimpleReporter;
use scylla::transport::session::{CurrentDeserializationApi, GenericSession, PoolSize};
use scylla::SessionBuilder;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

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
        default_value = "1000",
        help = "Number of concurrent requests at any given moment of time per executor",
    )]
    pub concurrency: usize,

    #[arg(
        short,
        long,
        default_value = "1",
        help = "Number of executors to run in parallel."
    )]
    pub executors_count: usize,

    #[arg(short, long, value_parser = parse, default_value = "10s", help = "Duration of the benchmark"
    )]
    pub duration: Duration,

    #[arg(
        short,
        long,
        default_value = "127.0.0.1:9042",
        help = "Comma-separated list of Scylla hosts with their ports. Example: '127.0.0.1:9042,127.0.0.2:9042,127.0.0.3:9042'"
    )]
    pub scylla_hosts: String,

    #[arg(
        short,
        long,
        default_value = "10",
        help = "Length of the key strings in the database"
    )]
    pub key_string_length: usize,

    #[arg(
        short,
        long,
        default_value = "10",
        help = "Size of the value blobs in the database"
    )]
    pub value_blob_size: usize,

    #[arg(
        short,
        long,
        default_value = "0.5",
        help = "Percentage of reads in the workload. The rest will be writes. Must be between 0.0 and 1.0"
    )]
    pub reads_percentage: f32,

    #[arg(
        short,
        long,
        default_value = "1000",
        help = "Total number of keys in the database"
    )]
    pub total_keys: usize,

    #[arg(short = 'u', long, default_value = "cassandra", help = "Scylla user")]
    pub user: String,

    #[arg(long, default_value = "cassandra", help = "Scylla password")]
    pub password: String,

    #[arg(
        short,
        long,
        default_value = "2",
        help = "Number of connections per shard in the connection pool"
    )]
    pub pool_size: usize,

    #[arg(
        short = 'm',
        long,
        help = "Available modes: simple, percentile. simple uses less cpu and memory, but provides less information. percentile uses more cpu and memory, but provides more information i.e. 50th, 90th, 99th percentiles",
        default_value = "simple"
    )]
    pub report_mode: String,

    #[arg(
        long,
        default_value = "1s",
        value_parser = parse,
        help = "Period of reporting results"
    )]
    pub report_period: Duration,

    #[arg(
        long,
        default_value = "true",
        help = "Drop the keyspace after the benchmark"
    )]
    pub dont_drop_test_keyspace: bool,
}

fn reporter_mode(mode: String, period: Duration) -> SimpleReporter {
    let reporter: SimpleReporter = match mode.as_str() {
        "simple" => SimpleReporter::new(period),
        // "percentile" => Box::new(PercentileReporter::new(period)),
        _ => panic!("Invalid mode: {}", mode),
    };
    reporter
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
        "Args: \
         duration: {}s, scylla_host: {}, pool_size: {},\n\
         user: {}, password: {}, key_string_length: {},\n\
         value_blob_size: {}, reads_percentage: {}, total_keys: {},\n\
         report_mode: {}, report_period: {}s, drop_test_keyspace: {}\
         executors: {}",
        args.duration.as_secs_f64(),
        args.scylla_hosts,
        args.pool_size,
        args.user,
        pass,
        args.key_string_length,
        args.value_blob_size,
        args.reads_percentage,
        args.total_keys,
        args.report_mode,
        args.report_period.as_secs_f64(),
        args.dont_drop_test_keyspace,
        args.executors_count
    );
    let hosts_split = args.scylla_hosts.split(",");
    let mut builder = SessionBuilder::new()
        .user(&args.user, &args.password)
        .pool_size(PoolSize::PerShard(
            NonZeroUsize::new(args.pool_size).unwrap(),
        ))
        .keyspaces_to_fetch(["test"]);
    for host in hosts_split {
        builder = builder.known_node(host);
    }
    let session: Arc<GenericSession<CurrentDeserializationApi>> = Arc::new(builder.build().await?);
    let reporter = Arc::new(reporter_mode(args.report_mode, args.report_period));
    let reporter_clone_for_thread = reporter.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(args.report_period).await;
            reporter_clone_for_thread.print_report();
        }
    });
    let mut handles = Vec::new();
    for i in 0..args.executors_count {
        let i_clone = i.clone();
        let reporter_clone = reporter.clone();
        let session_clone = session.clone();
        let handle = tokio::spawn(async move {
            let mut executor = executor::Executor::new(
                args.concurrency,
                args.key_string_length,
                args.value_blob_size,
                args.reads_percentage,
                args.total_keys,
                reporter_clone,
                args.dont_drop_test_keyspace,
            );
            let (stop_sender, executor_thread) = executor.start(session_clone).await.unwrap();
            tokio::time::sleep(args.duration).await;
            println!("Requesting stop since the duration has passed");
            if let Err(e) = stop_sender.send(()) {
                println!("Error sending stop signal: {:?}", e);
            }
            executor_thread.await.unwrap();
            println!("Executor #{} done", i_clone +1);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await?;
    }
    Ok(())
}
