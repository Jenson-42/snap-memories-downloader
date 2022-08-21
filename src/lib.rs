use pbr::ProgressBar;
use serde::Deserialize;
use std::{
    fs::OpenOptions,
    io::{prelude::*, stdin, ErrorKind, Write},
    path::Path,
    {env, thread, time},
};
mod config;
pub use crate::config::config::Config;

/// Memory
///
/// Struct to store information about snapchat memories as in the JSON file.
#[derive(Deserialize, Debug)]
struct Memory {
    date: String,
    media_type: String,
    download_link: String,
}

/// Run the snapchat memories downloader with a given config.
///
/// # Errors
/// Error messages are returned as a static string to show to the user.
///
/// # Panics
/// Panics if the working directory is empty or invalid.
pub fn run(config: Config) -> Result<(), &'static str> {
    let memories = match read_memories(&Path::new(&config.zip_path)) {
        Ok(m) => m,
        Err(e) => return Err(e),
    };

    //Allow user to back out before flooding their disk.
    println!(
        "\n{} memories will be downloaded into directory {}\\",
        memories.len(),
        env::current_dir()
            .expect("Working directory should be valid.")
            .join(&config.output_dir)
            .to_str()
            .expect("Path string should be valid.")
            .replace(r"\.\", r"\"),
    );
    println!("Are you sure you want to continue? [Y/N]");
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() == "y" {
        run_threads(memories, config);
    }

    Ok(())
}

/// Read the memories from the zipped json into a vector of memories.
///
/// # Errors
/// - Will return an error if the zip cannot be found or accessed.
/// - Will return an error if the archive doesn't contain "./json/memories_history.json".
/// - Will return an error if the JSON file can't be deserialized.
fn read_memories(zip_path: &Path) -> Result<Vec<Memory>, &'static str> {
    //Open .zip file.
    let zipfile = match std::fs::File::open(&zip_path) {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => return Err("Zip file not found."),
            ErrorKind::PermissionDenied => return Err("Permission to access zip file denied."),
            _ => return Err("Problem opening zip file."),
        },
    };

    //Read zip archive from file.
    let mut archive = match zip::ZipArchive::new(zipfile) {
        Ok(file) => file,
        Err(..) => {
            return Err("Problem reading zip file.");
        }
    };

    //Read JSON file from archive.
    let mut file = match archive.by_name("json/memories_history.json") {
        Ok(file) => file,
        Err(_) => {
            return Err("File json/memories_history.json not found in zip.");
        }
    };

    //Read JSON into a string.
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(_) => return Err("Problem reading JSON file."),
    }

    //Remove the outer dict of the json.
    let (contents, _) = contents.split_at(contents.len() - 1);
    let (_, contents) = contents.split_at(16);

    //Format the json to be deserialised.
    let json_str = contents
        .replace("Date", "date")
        .replace("Media Type", "media_type")
        .replace("Download Link", "download_link")
        .replace("Image", "image")
        .replace("Video", "video");

    //Deserialise the json into a vector of memories.
    let memories: Vec<Memory> = match serde_json::from_str(&json_str) {
        Ok(m) => m,
        Err(_) => return Err("Problem deserialising JSON file."),
    };

    return Ok(memories);
}

/// Struct to allow threads to communicate with main with an optional error message.
struct ThreadMessage {
    id: usize,
    message: Option<&'static str>,
}

impl ThreadMessage {
    fn no_error(id: usize) -> ThreadMessage {
        return ThreadMessage { id, message: None };
    }

    fn error(id: usize, message: &'static str) -> ThreadMessage {
        return ThreadMessage {
            id,
            message: Some(message),
        };
    }
}

fn run_threads(memories: Vec<Memory>, config: Config) -> () {
    //Store handles as vector of Options.
    //Handles are replaced with None values when finished.
    let mut thread_handles: Vec<Option<std::thread::JoinHandle<_>>> = Vec::new();
    //Sender and reciever objects allow threads to communicate with main.
    let (thread_sender, thread_reciever) = std::sync::mpsc::channel::<ThreadMessage>();

    //Only iterate through 100 items when in developer mode.
    let memories_iter = match config.developer_mode {
        true => memories[0..100].iter(),
        false => memories.iter(),
    };
    //Create a progress bar.
    let mut progressbar = ProgressBar::new(memories_iter.len() as u64);
    progressbar.format("╢▌▌░╟");
    progressbar.tick();

    for (i, memory) in memories_iter.enumerate() {
        //Sleep between requests as not to overload the server.
        thread::sleep(time::Duration::from_millis(config.thread_sleep as u64));

        let filename = create_filename(&memory, &config.output_dir);
        let download_link = memory.download_link.to_string();
        let thread_sender = thread_sender.clone();

        //Spawn a new thread to write the file to disk.
        thread_handles.push(Some(thread::spawn(move || {
            if Path::new(&filename).exists() {
                thread_sender
                    .send(ThreadMessage::error(i, "File already exists."))
                    .expect("Thread should be able to send message.");
                return ();
            }
            //Get the image bytes from snapchat.
            let image = match retrieve_image(&download_link) {
                Ok(image) => image,
                Err(e) => {
                    thread_sender
                        .send(ThreadMessage::error(i, e))
                        .expect("Thread should be able to send message.");
                    return ();
                }
            };
            //Write the bytes to file.
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(filename)
                .expect("File should be created and accessible.");
            file.write_all(&image)
                .expect("File should be accessible to write to.");

            //Send a message to main on completion.
            thread_sender.send(ThreadMessage::no_error(i)).unwrap();
        })));
    }

    //Create a string to store a list of errors.
    let mut thread_errors = String::new();

    loop {
        // Check if all threads are finished
        let num_left = thread_handles.iter().filter(|th| th.is_some()).count();
        if num_left == 0 {
            break;
        }

        //Wait until a thread is finished, then join it.
        let message = thread_reciever.recv().unwrap();
        let join_handle =
            std::mem::take(&mut thread_handles[message.id]).expect("Thread should still exist.");
        join_handle.join().unwrap();
        //Increment the progress bar.
        progressbar.inc();

        //If there is an error, append it to the errors string.
        if let Some(err) = message.message {
            let error = format!("\nThread {}: {}", message.id, err);
            thread_errors.push_str(&error)
        }
    }

    //Print relevant finished message.
    match thread_errors.is_empty() {
        true => println!("\n\nDownloading complete with 0 errors.\n"),
        false => println!("\n\nDownloading completed with errors:{}\n", thread_errors),
    }
}

///Create the filename from the date and media type of a memory.
fn create_filename(memory: &Memory, path: &str) -> String {
    let date = &memory.date.replace(":", "-");
    let extension = match memory.media_type.to_lowercase().as_ref() {
        "image" => "jpg",
        "video" => "mp4",
        _ => "unknown",
    };
    let filename = format!("{}/Memory {}.{}", path, date, extension);
    return filename;
}

#[tokio::main]
///Download an image from a snapchat link and return it as a vector of bytes.
async fn retrieve_image(link: &str) -> Result<Vec<u8>, &'static str> {
    let http_client = reqwest::Client::new();

    //First a POST request is made to snapchat to get the AWS link.
    let aws_link = match match http_client
        .post(link)
        .header("content-length", 0)
        .send()
        .await
    {
        Ok(resp) => resp,
        _ => return Err("Problem with snapchat response."),
    }
    .text()
    .await
    {
        Ok(url) => url,
        _ => return Err("Error retrieving AWS link from snapchat."),
    };

    //Then a GET request is made to the AWS link to retrieve the image bytes.
    let image = match match http_client
        .get(aws_link.to_string())
        .header("content-length", 0)
        .send()
        .await
    {
        Ok(resp) => resp,
        _ => return Err("Problem with AWS response."),
    }
    .bytes()
    .await
    {
        Ok(url) => url,
        _ => return Err("Error receiving image bytes."),
    };

    return Ok(image.to_vec());
}
