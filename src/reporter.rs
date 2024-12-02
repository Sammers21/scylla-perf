use std::time::Duration;

pub struct Reporter {}

impl Reporter {
    pub fn new() -> Reporter {
        Reporter {}
    }

    pub fn report_results(&self, latency: Duration) {
        println!("Reporting... latency: {:?}", latency);
    }
}
