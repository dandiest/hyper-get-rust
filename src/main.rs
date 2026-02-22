use http::header::CONTENT_LENGTH;
use reqwest::Client;
use std::error::Error;
use std::io::{self};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};

/// Standardizes user input by trimming whitespace and handling read errors.
fn inputs() -> String {
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(_) => buffer.trim().to_string(),
        Err(_) => {
            println!("Url not found / Invalid threads number.");
            String::new()
        }
    }
}

/// Retrieves the file size using an HTTP HEAD request.
/// If the server doesn't provide Content-Length via HEAD, it falls back to a ranged GET request.
async fn get_file_size(url: &str) -> Result<u64, Box<dyn std::error::Error>> {
    // We use a standard User-Agent to ensure compatibility with most servers/CDNs
    let client = Client::builder()
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
    .build()?;

    let url = url;

    // Attempt 1: HEAD request with extra headers
    let response = client.head(url).send().await?;

    if response.status().is_success() {
        if let Some(value) = response.headers().get(http::header::CONTENT_LENGTH) {
            return Ok(value.to_str()?.parse::<u64>()?);
        }
    }

    if let Some(value) = response.headers().get(CONTENT_LENGTH) {
        let size = value.to_str()?.parse::<u64>()?;
        return Ok(size);
    }

    // Attempt 2: GET request with Range 0-0 (Forces many stubborn servers to reveal size)
    let response = client
        .get(url)
        .header("Range", "bytes=0-0")
        .header("Accept", "*/*")
        .send()
        .await?;

    // Technic log: It says what's happening
    if !response.status().is_success() && response.status() != http::StatusCode::PARTIAL_CONTENT {
        return Err(format!("Server returned error status: {}", response.status()).into());
    }

    if response.status().is_success() || response.status() == http::StatusCode::PARTIAL_CONTENT {
        // Try Content-Length first (some servers return the full size here even for 0-0)
        if let Some(value) = response.headers().get(http::header::CONTENT_LENGTH) {
            let size = value.to_str()?.parse::<u64>()?;
            if size < 1 {
                return Ok(size);
            }
        }
    }

    // Try Content-Range (the most reliable for multi-thread detection)
    if let Some(content_range) = response.headers().get("content-range") {
        let range_str = content_range.to_str()?;
        if let Some(total_size_str) = range_str.split('/').last() {
            let size = total_size_str.parse::<u64>()?;
            return Ok(size);
        }
    }

    Err("The server refused to provide file size or doesn't support ranged requests.".into())
}

/// Downloads a specific byte range and writes it to the correct position in the file.
async fn download_apart(
    url: String,
    start: u64,
    end: u64,
    path: Arc<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let range_header = format!("bytes={}-{}", start, end);

    // Request only the specified range
    let response = client.get(url).header("Range", range_header).send().await?;
    let data = response.bytes().await?;

    // Open file with write permissions; path is wrapped in Arc for thread-safety
    let mut file = OpenOptions::new().write(true).open(&*path).await?;

    // Seek to the correct offset before writing to ensure data integrity
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

    // Share ownership of the path string across all spawned tasks
    let path_arc = Arc::new(path);

    // Pre-allocate the file on disk to prevent fragmentation
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

        // Spawn independent asynchronous tasks for parallel downloading
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

    // Wait for all parallel tasks to complete
    for h in handles {
        h.await?;
    }

    println!("Download successfully completed.");
    Ok(())
}
