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

/// Retrieves the file size via an HTTP HEAD request to avoid downloading the body.
async fn get_file_size(url: &str) -> Result<u64, Box<dyn std::error::Error>> {      
    let client = Client::new();
    let url = url;
    let response = client.head(url).send().await?;
    
    // Extract Content-Length header and parse it as u64
    let c_len = response.headers().get(CONTENT_LENGTH);

    if let Some(value) = c_len {                                                    
        let text = value.to_str()?;
        let real_value: u64 = text.parse::<u64>()?;
        println!("Real content length: {:?}", value);
        Ok(real_value)
    } else {                                                                          
        Err("Content length not found.".into())
    }
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
