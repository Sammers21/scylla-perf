# Scylla-perf

Build:
```bash
$ cargo build --release
```

Usage example (percentile mode):
```
$ ./target/release/scylla-perf  -m percentile

Args: duration: 10s, scylla_host: 127.0.0.1:9042, pool_size: 2, user: cassandra, password: cass*****, key_string_length: 10, value_blob_size: 10, reads_percentage: 0.5, total_keys: 1000, report_mode: percentile, report_period: 1, drop_test_keyspace: true
Generating 1000 key-value pairs, key length: 10, value size: 10
Starting executor...
Inserting initial key-value pairs...
Done inserting initial key-value pairs
Starting queries...

┌────────────┬───────────┬───────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
│ Query Type ┆ Count     ┆ RPS           ┆ Latency p50 ┆ Latency p75 ┆ Latency p95 ┆ Latency p99 │
╞════════════╪═══════════╪═══════════════╪═════════════╪═════════════╪═════════════╪═════════════╡
│ Total      ┆ 147.555 K ┆ 20.13 K req/s ┆ 26.43 ms    ┆ 59.77 ms    ┆ 132.61 ms   ┆ 202.24 ms   │
├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Read       ┆ 73.491 K  ┆ 10.03 K req/s ┆ 26.43 ms    ┆ 59.77 ms    ┆ 131.84 ms   ┆ 202.24 ms   │
├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Write      ┆ 74.064 K  ┆ 10.10 K req/s ┆ 26.43 ms    ┆ 59.77 ms    ┆ 132.61 ms   ┆ 202.24 ms   │
└────────────┴───────────┴───────────────┴─────────────┴─────────────┴─────────────┴─────────────┘

```

Usage example (simple mode):
```
$ ./target/release/scylla-perf          
Args: duration: 10s, scylla_host: 127.0.0.1:9042, pool_size: 2, user: cassandra, password: cass*****, key_string_length: 10, value_blob_size: 10, reads_percentage: 0.5, total_keys: 1000, report_mode: simple, report_period: 1, drop_test_keyspace: true
Generating 1000 key-value pairs, key length: 10, value size: 10
Starting executor...
Inserting initial key-value pairs...
Done inserting initial key-value pairs
Starting queries...
Total: 4296 reqs, 4270.03 req/s, avg latency: 8.39 ms
Total: 20528 reqs, 10177.11 req/s, avg latency: 40.76 ms
Total: 43931 reqs, 14556.77 req/s, avg latency: 37.68 ms
Total: 64661 reqs, 16050.46 req/s, avg latency: 37.71 ms
Total: 87198 reqs, 17332.53 req/s, avg latency: 36.38 ms
Total: 110732 reqs, 18338.45 req/s, avg latency: 35.43 ms
Total: 128147 reqs, 18206.31 req/s, avg latency: 37.08 ms
Total: 147816 reqs, 18379.11 req/s, avg latency: 37.37 ms
Total: 170639 reqs, 18868.88 req/s, avg latency: 36.98 ms
Total: 205847 reqs, 20495.50 req/s, avg latency: 32.65 ms
Coordinator received stop signal, waiting for concurrent tasks to finish...
All tasks finished, stopping metrics reporter...
Dropping test keyspace...
Stopping metrics reporter...
Dropped test keyspace

```

Avialable options:
```
$ ./target/release/scylla-perf --help 
ScyllaDB performance tool

Usage: scylla-perf [OPTIONS]

Options:
  -c, --concurrency <CONCURRENCY>
          Number of concurrent requests at any given moment of time [default: 1000]
  -d, --duration <DURATION>
          Duration of the benchmark [default: 10s]
  -s, --scylla-hosts <SCYLLA_HOSTS>
          Comma-separated list of Scylla hosts with their ports. Example: '127.0.0.1:9042,127.0.0.2:9042,127.0.0.3:9042' [default: 127.0.0.1:9042]
  -k, --key-string-length <KEY_STRING_LENGTH>
          Length of the key strings in the database [default: 10]
  -v, --value-blob-size <VALUE_BLOB_SIZE>
          Size of the value blobs in the database [default: 10]
  -r, --reads-percentage <READS_PERCENTAGE>
          Percentage of reads in the workload. The rest will be writes. Must be between 0.0 and 1.0 [default: 0.5]
  -t, --total-keys <TOTAL_KEYS>
          Total number of keys in the database [default: 1000]
  -u, --user <USER>
          Scylla user [default: cassandra]
      --password <PASSWORD>
          Scylla password [default: cassandra]
  -p, --pool-size <POOL_SIZE>
          Number of connections per shard in the connection pool [default: 2]
  -m, --report-mode <REPORT_MODE>
          Available modes: simple, percentile. simple uses less cpu and memory, but provides less information. percentile uses more cpu and memory, but provides more information i.e. 50th, 90th, 99th percentiles [default: simple]
      --report-period <REPORT_PERIOD>
          Period of reporting results [default: 1s]
      --drop-test-keyspace
          Drop the keyspace after the benchmark
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version

```