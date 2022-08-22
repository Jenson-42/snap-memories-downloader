use snapchat_memories_downloader::{run, Config};
use std::process;

#[tokio::main]
async fn main() -> () {
    println!("\nSnapchat Memories Downloader V0.1.0");

    //Get configuration from command line arguments.
    let config = Config::from_args();

    //Run the program with the given configuration.
    if let Err(e) = run(config) {
        println!("Application error: {e}");

        process::exit(1);
    }
}
