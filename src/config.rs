pub mod config {
    use clap::Parser;
    use std::env;

    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about = None)]
    pub struct Config {
        ///Path to the "mydata" zip file.
        #[clap(short, long, value_parser)]
        pub zip_path: String,

        ///Path to the output directory.
        #[clap(short, long, value_parser)]
        pub output_dir: String,

        //How long to sleep between threads to avoid getting rate-limited.
        #[clap(short, long, value_parser, default_value_t = 1)]
        pub thread_sleep: i32,

        ///Whether to start in developer mode (only downloads first 100 memories).
        #[clap(short, long, value_parser, default_value_t = false)]
        pub developer_mode: bool,
    }

    impl Config {
        pub fn from_args() -> Config {
            let mut config = Config::parse();

            //Developer mode environment variable for quick tests.
            //Environment variable will overwrite CLI argument.
            if env::var("DOWNLOADER_TEST_MODE").is_ok() {
                println!("WARNING: DEVELOPER MODE ACTIVE. Only 100 memories will be downloaded.");
                config.developer_mode = true;
            };

            config
        }
    }
}
