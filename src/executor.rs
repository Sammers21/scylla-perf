use crate::reporter::Reporter;
pub struct Executor {
    session: scylla::Session,
    concurrency: u32,
    reporter: Reporter,
}

impl Executor {
    pub fn new(session: scylla::Session, concurrency: u32, reporter: Reporter) -> Executor {
        Executor {
            session,
            concurrency,
            reporter,
        }
    }

    pub fn start(&self) -> () {
        println!("Starting executor...");
    }

    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Stopping executor...");
        Ok(())
    }
}
