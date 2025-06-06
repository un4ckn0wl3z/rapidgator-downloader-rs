use colored::Colorize;
use downloader::download_file;
use indicatif::MultiProgress;
use models::ResponseData;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::Semaphore;

use crate::rg_endpoint::RG_LOGIN_URL;

mod downloader;
mod models;
mod rg_endpoint;
mod url_parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let banner = r#"
   ___  _____    ___  ____ _      ___  ____   ____  ___   ___  _______ 
  / _ \/ ___/___/ _ \/ __ \ | /| / / |/ / /  / __ \/ _ | / _ \/ __/ _ \
 / , _/ (_ /___/ // / /_/ / |/ |/ /    / /__/ /_/ / __ |/ // / _// , _/
/_/|_|\___/   /____/\____/|__/|__/_/|_/____/\____/_/ |_/____/___/_/|_| 
                                                                       
    Developer: un4ckn0wl3z
    https://github.com/un4ckn0wl3z 
    "#
    .green();
    println!(
        "{banner} version: {}{} \n",
        "v".green(),
        env!("CARGO_PKG_VERSION").green()
    );

    // Load credentials from YAML file
    let credentials_file = File::open("config.yaml").expect(
        "config.yaml is missing. Please create the file and include your email and password.",
    );
    let credentials: crate::models::Credentials =
        serde_yaml::from_reader(credentials_file).expect("Cannot parse email or password.");
    let login = credentials.login;
    let password = credentials.password;
    let max_concurrent_downloads = credentials.max_concurrent_downloads;
    let target_dowload_path = credentials.target_dowload_path;

    let mut login_params = HashMap::new();
    login_params.insert("login", login);
    login_params.insert("password", password);

    let client = reqwest::Client::new();
    let mp = MultiProgress::new(); // Create MultiProgress instance

    let login_response = client.get(RG_LOGIN_URL).query(&login_params).send().await?;

    if login_response.status().is_success() {
        let login_response_deserialized: ResponseData =
            serde_json::from_str(&login_response.text().await?)
                .expect("Cannot parse email or password from login response. Maybe login failed.");
        // println!("Token: {:?}", login_response_deserialized.response.token);

        let file_list = "files.txt";
        let file = File::open(file_list)
            .expect("files.txt is missing. Please create the file and include your download link");
        let reader = io::BufReader::new(file);

        let semaphore = Arc::new(Semaphore::new(max_concurrent_downloads)); // Limit concurrent tasks
        let mut handles = vec![];
        let has_failed = Arc::new(AtomicBool::new(false));

        // Iterate over lines in the file and spawn async tasks
        for line_result in reader.lines() {
            let line = line_result?;
            // println!("Processing URL: {}", line);
            let client = client.clone();
            let token = login_response_deserialized.response.token.clone();
            let url = line.clone();
            let mp = mp.clone();
            let permit = semaphore.clone();
            let has_failed = Arc::clone(&has_failed);
            let target_dowload_path = target_dowload_path.clone();

            // Spawn an async task for each file download
            let handle = tokio::spawn(async move {
                // Acquire a permit
                let _permit = permit.acquire().await.unwrap();
                if let Err(e) = download_file(target_dowload_path, client, token, url, mp).await {
                    println!("Error downloading file: {}", e);
                    has_failed.store(true, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await?;
        }
        println!(
            "\n{}",
            if !has_failed.load(Ordering::SeqCst) {
                "All files have been downloaded.".green()
            } else {
                "Not all files were downloaded; some succeeded. Please check error.log for more detail.".red()
            }
        );
    } else {
        panic!("Failed to login: {}", login_response.status());
    }

    Ok(())
}
