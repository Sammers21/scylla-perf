# Scylla-perf

Build:
```bash
$ cargo build --release
```

Usage example:
```
 ./target/release/scylla-perf       
Args: duration: 10s, scylla_host: 127.0.0.1:9042, user: cassandra, password: cass*****, key_string_length: 100, value_blob_size: 100, reads_percentage: 0.5, total_keys: 1000
Generating 1000 key-value pairs, key length: 100, value size: 100
Starting executor...
Inserting initial key-value pairs...
Done inserting initial key-value pairs
Starting queries...
┌────────────┬──────────┬───────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
│ Query Type ┆ Count    ┆ RPS           ┆ Latency p50 ┆ Latency p75 ┆ Latency p95 ┆ Latency p99 │
╞════════════╪══════════╪═══════════════╪═════════════╪═════════════╪═════════════╪═════════════╡
│ Total      ┆ 2.010 K  ┆ 1.97 K req/s  ┆ 52.86 ms    ┆ 93.57 ms    ┆ 185.60 ms   ┆ 243.71 ms   │
├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Read       ┆ 978.000  ┆ 960.06  req/s ┆ 54.40 ms    ┆ 93.95 ms    ┆ 183.29 ms   ┆ 231.42 ms   │
├╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Write      ┆ 1.032 K  ┆ 1.01 K req/s  ┆ 50.17 ms    ┆ 93.57 ms    ┆ 194.05 ms   ┆ 246.78 ms   │
└────────────┴──────────┴───────────────┴─────────────┴─────────────┴─────────────┴─────────────┘
```

Avialable options:
```
$ ./target/release/scylla-perf --help 
A tool for benchmarking Scylla DB performance

Usage: scylla-perf [OPTIONS]

Options:
  -c, --concurrency <CONCURRENCY>
          Number of concurrent requests at any given moment of time
          
          [default: 100]

  -d, --duration <DURATION>
          Duration of the benchmark
          
          [default: 10s]

  -s, --scylla-hosts <SCYLLA_HOSTS>
          Comma-separated list of Scylla hosts with their ports. Example: '127.0.0.1:9042,127.0.0.2:9042,127.0.0.3:9042'
          
          [default: 127.0.0.1:9042]

  -k, --key-string-length <KEY_STRING_LENGTH>
          Length of the key strings in the database
          
          [default: 100]

  -v, --value-blob-size <VALUE_BLOB_SIZE>
          Size of the value blobs in the database
          
          [default: 100]

  -r, --reads-percentage <READS_PERCENTAGE>
          Percentage of reads in the workload. The rest will be writes. Must be between 0.0 and 1.0
          
          [default: 0.5]

  -t, --total-keys <TOTAL_KEYS>
          Total number of keys in the database
          
          [default: 1000]

  -u, --user <USER>
          Scylla user
          
          [default: cassandra]

      --password <PASSWORD>
          Scylla password
          
          [default: cassandra]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```