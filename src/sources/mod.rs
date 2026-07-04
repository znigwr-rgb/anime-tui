pub mod nyaa;
pub mod subsplease;

#[derive(Debug, Clone)]
pub struct TorrentEntry {
    pub title: String,
    pub magnet: String,
    pub size: String,
    pub seeders: u32,
    pub leechers: u32,
    pub source: &'static str,
}

impl TorrentEntry {
    pub fn fmt_size(&self) -> &str {
        &self.size
    }

    pub fn fmt_source(&self) -> &str {
        self.source
    }
}
