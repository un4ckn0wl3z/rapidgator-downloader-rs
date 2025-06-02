use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    pub response: InnerResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileDownloadResponseData {
    pub response: InnerFileDownloadResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerFileDownloadResponseData {
    pub download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    pub login: String,
    pub password: String,
    pub max_concurrent_downloads: usize,
}
