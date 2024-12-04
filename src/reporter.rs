use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use histogram::Histogram;
use human_format::Formatter;
use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;
use tokio::time::Instant;

pub trait Reporter {
    /// Create a new reporter with a given period between reports
    fn new(period: Duration) -> Self
    where
        Self: Sized;
    fn report_results(&mut self, query_type: QueryType, time: Duration) -> ();
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug, Ord, PartialOrd)]
pub enum QueryType {
    Total,
    Read,
    Write,
}

#[derive(Clone)]
pub struct SimpleReporter {
    period: Duration,
    request_counts: usize,
    request_durations: Duration,
    first_reported_at: Instant,
    last_reported_at: Instant,
}

#[derive(Clone)]
pub struct PercentileReporter {
    period: Duration,
    request_counts: BTreeMap<QueryType, usize>,
    request_durations: BTreeMap<QueryType, Histogram>,
    first_reported_at: Instant,
    last_reported_at: Instant,
}

unsafe impl Send for SimpleReporter {

}

unsafe impl Send for PercentileReporter {

}

impl Reporter for SimpleReporter {
    fn new(period: Duration) -> Self {
        SimpleReporter {
            period,
            request_counts: 0,
            request_durations: Duration::from_secs(0),
            last_reported_at: Instant::now(),
            first_reported_at: Instant::now(),
        }
    }

    fn report_results(&mut self, _: QueryType, latency: Duration) -> () {
        self.request_counts += 1;
        self.request_durations += latency;
        if self.last_reported_at.elapsed() > self.period {
            let rps = self.request_counts as f64 / self.first_reported_at.elapsed().as_secs_f64();
            let avg_latency = self.request_durations.as_secs_f64() / self.request_counts as f64;
            println!(
                "Total: {} reqs, {:.2} req/s, avg latency: {:.2} ms",
                self.request_counts,
                rps,
                avg_latency * 1000.0
            );
            self.last_reported_at = Instant::now();
        }
    }
}

impl PercentileReporter {
    fn colored_row(row: Vec<String>, color: Color) -> Vec<Cell> {
        row.into_iter().map(|s| Cell::new(s).fg(color)).collect()
    }

    fn add_percentile(hist: &Histogram, percentile: f64, row: &mut Vec<String>) {
        let start = hist.percentile(percentile).unwrap().unwrap().start();
        let end = hist.percentile(percentile).unwrap().unwrap().end();
        let between = end + start / 2;
        row.push(format!("{:.2} ms", between as f64 / 1000.0));
    }
}

impl fmt::Display for QueryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Reporter for PercentileReporter {
    fn new(period: Duration) -> Self {
        PercentileReporter {
            period,
            request_counts: BTreeMap::new(),
            request_durations: BTreeMap::new(),
            last_reported_at: Instant::now(),
            first_reported_at: Instant::now(),
        }
    }

    fn report_results(&mut self, query_type: QueryType, latency: Duration) -> () {
        for qt in &vec![QueryType::Total, query_type] {
            let count = self.request_counts.entry(qt.clone()).or_insert(0);
            *count += 1;
            let hist = self
                .request_durations
                .entry(qt.clone())
                .or_insert(Histogram::new(7, 64).unwrap());
            let latency_as_u64 = (latency.as_secs_f64() * 1000000.0) as u64;
            let res = hist.add(latency_as_u64, 1);
            if res.is_err() {
                println!("Failed to add latency to histogram");
            }
        }
        if self.last_reported_at.elapsed() > self.period {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic);
            let mut header = Vec::new();
            header.push("Query Type");
            header.push("Count");
            header.push("RPS");
            header.push("Latency p50");
            header.push("Latency p75");
            header.push("Latency p95");
            header.push("Latency p99");
            table.set_header(header);
            for (query_type, count) in &self.request_counts {
                let hist = self.request_durations.get(query_type).unwrap();
                let mut row = Vec::new();
                row.push(format!("{:?}", query_type));
                row.push(Formatter::new().with_decimals(3).format(*count as f64));
                let rps = *count as f64 / self.first_reported_at.elapsed().as_secs_f64();
                row.push(Formatter::new().format(rps) + " req/s");
                Self::add_percentile(hist, 50.0, &mut row);
                Self::add_percentile(hist, 75.0, &mut row);
                Self::add_percentile(hist, 95.0, &mut row);
                Self::add_percentile(hist, 99.0, &mut row);
                if *query_type == QueryType::Total {
                    table.add_row(Self::colored_row(row, Color::Green));
                } else {
                    table.add_row(row);
                }
            }
            println!("{table}\n");
            self.last_reported_at = Instant::now();
        }
    }
}
