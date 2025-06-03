use crate::url_parser::extract_file_info;
use crate::{models::FileDownloadResponseData, rg_endpoint::RG_DOWNLOAD_URL};
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{collections::HashMap, fs::File, io::Write};

pub async fn download_file(
    client: reqwest::Client,
    token: String,
    url: String,
    mp: MultiProgress,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Helper function to log errors to error.log
    fn log_error_to_file(error_msg: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("error.log")?;
        writeln!(file, "{}: [DOWNLOAD_FAIL] -> {}", chrono::Local::now().to_rfc3339(), error_msg)?;
        Ok(())
    }

    if let Ok((file_id, filename)) = extract_file_info(url.as_str()) {
        let mut file_download_params = HashMap::new();
        file_download_params.insert("file_id", file_id);
        file_download_params.insert("token", token);

        let file_download_response = client
            .get(RG_DOWNLOAD_URL)
            .query(&file_download_params)
            .send()
            .await?;

        if file_download_response.status().is_success() {
            let login_response_deserialized: FileDownloadResponseData =
                match serde_json::from_str(&file_download_response.text().await?) {
                    Ok(data) => data,
                    Err(e) => {
                        let error_msg =
                            format!("Failed to deserialize response for {}: {}", url, e);
                        log_error_to_file(&error_msg)?;
                        println!("{}", error_msg.red());
                        return Err(e.into());
                    }
                };

            let download_url = &login_response_deserialized.response.download_url;
            let response = client.head(download_url).send().await?;
            let total_size = response
                .headers()
                .get("content-length")
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse().ok())
                .unwrap_or(0);

            // Create progress bar and add to MultiProgress
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
                    .progress_chars("#>-"),
            );
            pb.set_message(format!("Downloading {}", url));
            let pb = mp.add(pb);

            // Send GET request to download the file
            let mut response = client.get(download_url).send().await?;
            let mut file = match File::create(&filename) {
                Ok(file) => file,
                Err(e) => {
                    let error_msg = format!("Failed to create file {}: {}", filename, e);
                    log_error_to_file(&error_msg)?;
                    println!("{}", error_msg.red());
                    return Err(e.into());
                }
            };

            // Download and write file with progress updates
            while let Some(chunk) = response.chunk().await? {
                file.write_all(&chunk)?;
                pb.inc(chunk.len() as u64);
            }

            // Finish the progress bar
            pb.finish_with_message("Download completed");
        } else {
            let error_msg = format!("Cannot get file download URL for {}", url);
            log_error_to_file(&error_msg)?;
            println!("{}", error_msg.red());
            return Err("Cannot get file download URL".into());
        }
    } else {
        let error_msg = format!("Cannot extract file info for {}", url);
        log_error_to_file(&error_msg)?;
        println!("{}", error_msg.red());
        return Err("Cannot extract file info".into());
    }
    Ok(())
}
