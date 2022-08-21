use snapchat_memories_downloader::{run, Config};
use std::{env, process};

#[tokio::main]
async fn main() -> () {
    println!("\nSnapchat Memories Downloader V0.1");

    let config = Config::build(env::args()).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1)
    });

    if let Err(e) = run(config) {
        println!("Application error: {e}");

        process::exit(1);
    }
}
