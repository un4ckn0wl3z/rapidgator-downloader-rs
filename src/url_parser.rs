use std::path::Path;
use url::Url;

pub fn extract_file_info(url_str: &str) -> Result<(String, String), &'static str> {
    let url = Url::parse(url_str).map_err(|_| "Invalid URL")?;
    let segments: Vec<&str> = url.path_segments().ok_or("No path segments")?.collect();

    if segments.len() >= 2 && segments[0] == "file" {
        let file_id = segments[1].to_string();
        let filename = Path::new(segments[2])
            .file_stem()
            .ok_or("Invalid filename")?
            .to_str()
            .ok_or("Invalid filename encoding")?
            .to_string();
        Ok((file_id, filename))
    } else {
        Err("URL does not match expected RapidGator format")
    }
}
