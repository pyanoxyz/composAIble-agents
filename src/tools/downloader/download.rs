use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::fs::create_dir_all;
use indicatif::{ ProgressBar, ProgressStyle };
use futures_util::StreamExt;

pub async fn download_model_files(
    model_path: &str,
    save_dir: &str
) -> Result<(), Box<dyn std::error::Error>> {
    create_dir_all(save_dir).await?;
    let client = Client::new();
    let response = client.get(model_path).send().await?;

    if response.status().is_success() {
        let file_name = model_path.split('/').last().unwrap_or("model");
        let file_path = Path::new(save_dir).join(file_name);
        let mut file = File::create(file_path)?;

        let total_size = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        let style = ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            )?
            .progress_chars("#>-");
        pb.set_style(style);

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message("Download complete");
        println!("Model downloaded successfully to {}", save_dir);
    } else {
        eprintln!("Failed to download model: {}", response.status());
    }

    Ok(())
}
