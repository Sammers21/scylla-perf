use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {

    #[arg(short, long)]
    concurrency: i32,

    #[arg(short, long)]
    proportion: str,

    #[arg(short, long)]
    duration: str,

    #[arg(short, long)]
    size: str,
}

struct Params {
    concurrency: i32,
    writesPercentage: f32,
    readsPercentage: f32,
    duration: str,
    size: str,
}

fn main() {
    let args = Cli::parse();
    println!("{:?}", args);

}
