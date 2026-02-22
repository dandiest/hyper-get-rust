use http::header::CONTENT_LENGTH;
use reqwest::Client;
use std::error::Error;
use std::io::{self};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};

fn inputs() -> String {                                                            /// Input function used to avoid code repetition
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(_) => buffer.trim().to_string(),
        Err(_) => {
            println!("Url not found / Invalid threads number.");
            String::new()
        }
    }
}

async fn get_file_size(url: &str) -> Result<u64, Box<dyn std::error::Error>> {      /// Fetches the total size of the remote file using a HEAD request.
    let client = Client::new();
    let url = url;
    let response = client.head(url).send().await?;
    let c_len = response.headers().get(CONTENT_LENGTH);

    if let Some(value) = c_len {                                                     /// Downloads a specific byte range of the file and writes it to the corresponding offset on disk.       
        let text = value.to_str()?;
        let real_value: u64 = text.parse::<u64>()?;
        println!("Real content length: {:?}", value);
        Ok(real_value)
    } else {                                                                          // Else, it returns an error
        Err("Content length not found.".into())
    }
}

async fn download_apart(                                                              // An asynchronous function that allows you to download a file via a URL much faster than from a browser,
    url: String,                                                                      // dividing the download into multiple parts.
    start: u64,
    end: u64,
    path: Arc<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let range_header = format!("bytes={}-{}", start, end);

    let response = client.get(url).header("Range", range_header).send().await?;

    let data = response.bytes().await?;

    let mut file = OpenOptions::new().write(true).open(&*path).await?;

    file.seek(SeekFrom::Start(start)).await?;
    file.write_all(&data).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Please, insert the url.");
    let url_input = inputs();
    println!("How many threads do you want to use?");
    let threads_input = inputs();
    let n_threads: usize = match threads_input.trim().parse() {
        Ok(num) => {
            println!("Valid threads number: {}", num);
            num
        }
        Err(_) => {
            return Err("Invalid input, please retry.".into());
        }
    };

    let total = get_file_size(&url_input).await?;
    let slice_weight = total / n_threads as u64;
    println!("Please, insert the path.");
    let path_input = inputs();
    let path = match path_input.as_str() {
        "" => {
            println!("No path inserted. Using default: download.bin");
            "download.bin".to_string()
        }
        valid_name => {
            println!("File will be saved as: {}", valid_name);
            valid_name.to_string()
        }
    };

    let path_arc = Arc::new(path);

    let first_file = tokio::fs::File::create(&*path_arc).await?;
    first_file.set_len(total).await?;

    let mut handles = vec![];
    for i in 0..n_threads {
        let start = i as u64 * slice_weight;
        let end = if i == n_threads - 1 {
            total - 1
        } else {
            start + slice_weight - 1
        };
        let url_clone = url_input.to_string();
        let path_clone = Arc::clone(&path_arc);
        let handle = tokio::spawn(async move {
            let download = download_apart(url_clone, start as u64, end, path_clone).await;
            match download {
                Ok(_) => {
                    println!("Thread downloaded successfully.");
                }
                Err(_) => {
                    println!("The thread failed.");
                }
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.await?;
    }

    println!("Download successfully completed.");
    Ok(())
}
