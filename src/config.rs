pub mod config {
    use std::env;

    /// Config
    ///
    /// A struct to store configuration settings for the application.
    pub struct Config {
        pub zip_path: String,
        pub output_dir: String,
        pub thread_sleep: i32,
        pub developer_mode: bool,
    }

    impl Config {
        pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
            //Ignore first argument as it is the executable name.
            args.next();

            //Path to the "mydata" zip file from snapchat.
            let zip_path = match args.next() {
                Some(arg) => arg,
                None => return Err("Didn't get a zip path."),
            };

            //Output directory for image and video files.
            let output_dir = match args.next() {
                Some(arg) => arg,
                None => return Err("Didn't get an output directory."),
            };

            //How long to sleep between creation of each thread in ms.
            let thread_sleep: i32 = match args.next() {
                Some(arg) => match arg.parse::<i32>() {
                    Ok(num) => num,
                    Err(_) => return Err("Thread sleep must be a valid integer."),
                },
                None => 100, //Default value is 100ms
            };

            //Developer mode environment variable for quick tests.
            let developer_mode = match env::var("DOWNLOADER_TEST_MODE") {
                Ok(_) => {
                    println!(
                        "WARNING: DEVELOPER MODE ACTIVE. Only 100 memories will be downloaded."
                    );
                    true
                }
                _ => false,
            };

            Ok(Config {
                zip_path,
                output_dir,
                thread_sleep,
                developer_mode,
            })
        }
    }
}
