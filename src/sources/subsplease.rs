use super::TorrentEntry;

#[derive(serde::Deserialize, Debug)]
struct SpEntry {
    #[serde(default)]
    show: Option<String>,
    #[serde(default)]
    episode: Option<String>,
    #[serde(default)]
    downloads: Vec<SpDownload>,
}

#[derive(serde::Deserialize, Debug)]
struct SpDownload {
    #[serde(default)]
    res: Option<String>,
    #[serde(default)]
    magnet: Option<String>,
}

pub async fn search(query: &str) -> Result<Vec<TorrentEntry>, String> {
    let url = format!(
        "https://subsplease.org/api/?f=search&s={}",
        urlencoding(query)
    );

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("SubsPlease: {}", e))?;

    let status = resp.status();
    let body = resp.text().await.map_err(|e| format!("SubsPlease: {}", e))?;

    if !status.is_success() || body.trim().is_empty() {
        return Ok(Vec::new());
    }

    let json: Result<serde_json::Value, _> = serde_json::from_str(&body);
    match json {
        Ok(serde_json::Value::Object(map)) => {
            let mut entries = Vec::new();
            for (_key, val) in &map {
                if let Some(entry) = serde_json::from_value::<SpEntry>(val.clone()).ok() {
                    let best = entry
                        .downloads
                        .iter()
                        .find(|d| d.magnet.is_some())
                        .or_else(|| entry.downloads.first());

                    if let Some(dl) = best {
                        if let Some(magnet) = &dl.magnet {
                            let title = format!(
                                "{} {} [{}p]",
                                entry.show.as_deref().unwrap_or("Unknown"),
                                entry.episode.as_deref().unwrap_or(""),
                                dl.res.as_deref().unwrap_or("?")
                            );

                            entries.push(TorrentEntry {
                                title,
                                magnet: magnet.clone(),
                                size: String::new(),
                                seeders: 0,
                                leechers: 0,
                                source: "SubsPlease",
                            });
                        }
                    }
                }
            }
            Ok(entries)
        }
        _ => Ok(Vec::new()),
    }
}

fn urlencoding(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join("%20")
}
