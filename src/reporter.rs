use std::time::Duration;
use tokio::time::Instant;

pub struct Reporter {
    total_requests: usize,
    total_duration: Duration,
    first_reported_at: Instant,
    last_reported_at: Instant
}

impl Reporter {
    pub fn new() -> Reporter {
        Reporter {
            total_requests: 0,
            total_duration: Duration::from_secs(0),
            last_reported_at: Instant::now(),
            first_reported_at: Instant::now()
        }
    }

    pub fn report_results(&mut self, latency: Duration) {
        self.total_duration += latency;
        self.total_requests += 1;
        if self.last_reported_at.elapsed() > Duration::from_secs(1) {
            let rps = self.total_requests as f64 / self.first_reported_at.elapsed().as_secs_f64();
            let avg_latency = self.total_duration.as_secs_f64() / self.total_requests as f64;
            println!("RPS: {:.2}, Avg. Latency: {:.2} ms", rps, avg_latency * 1000.0);
            self.last_reported_at = Instant::now();
        }
    }
}
