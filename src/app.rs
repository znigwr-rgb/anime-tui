use crate::download;
use crate::sources::{self, TorrentEntry};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Searching,
}

pub struct App {
    pub input_mode: InputMode,
    pub query: String,
    pub results: Vec<TorrentEntry>,
    pub selected: usize,
    pub status: String,
    pub loading: bool,
    pub scroll_offset: usize,
    pub spanish_filter: bool,
    last_search: Option<String>,
    last_search_time: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            input_mode: InputMode::Searching,
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            status: "Type to search, Enter to search, 'd' download (aria2c), 's' Spanish, 'q' quit".into()
            loading: false,
            scroll_offset: 0,
            spanish_filter: false,
            last_search: None,
            last_search_time: Instant::now(),
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
    }

    pub fn previous(&mut self) {
        if !self.results.is_empty() {
            if self.selected > 0 {
                self.selected -= 1;
            }
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    pub fn next(&mut self) {
        if !self.results.is_empty() {
            let max = self.results.len() - 1;
            if self.selected < max {
                self.selected += 1;
            }
            if self.selected >= self.scroll_offset + 15 {
                self.scroll_offset = self.selected - 15 + 1;
            }
        }
    }

    pub async fn search(&mut self) {
        let query = self.query.trim().to_string();
        if query.is_empty() {
            return;
        }

        self.loading = true;
        self.results.clear();
        self.selected = 0;
        self.scroll_offset = 0;
        self.status = format!("Searching for \"{}\"...", query)

        let mut all = Vec::new();

        let (nyaa_res, subs_res) = tokio::join!(
            sources::nyaa::search(&query, self.spanish_filter),
            sources::subsplease::search(&query),
        );

        if let Ok(entries) = nyaa_res {
            all.extend(entries);
        }

        if let Ok(entries) = subs_res {
            all.extend(entries);
        }

        all.sort_by(|a, b| b.seeders.cmp(&a.seeders));

        self.results = all;

        if self.results.is_empty() {
            self.status = "No results found. Try a different search.".into()
        } else {
            self.status = format!(
                "Found {} result(s). Use ↑/↓ to navigate, 'd' to download, Enter to search again",
                self.results.len()
            );
        }

        self.loading = false;
        self.last_search = Some(query);
        self.last_search_time = Instant::now();
    }

    pub fn get_selected_magnet(&self) -> Option<String> {
        self.results.get(self.selected).map(|e| e.magnet.clone())
    }

    pub fn toggle_spanish(&mut self) {
        self.spanish_filter = !self.spanish_filter;
        if self.spanish_filter {
            self.status = "Spanish filter ON — searching Latino/Castellano dubs".into()
        } else {
            self.status = "Spanish filter OFF".into()
        }
    }

    pub fn download_selected(&mut self) {
        let magnet = match self.get_selected_magnet() {
            Some(m) => m,
            None => {
                self.status = "No torrent selected".into();
                return
            }
        };

        let title = self.results[self.selected].title.clone();

        match download::start_download(&magnet) {
            Ok(dl) => {
                self.status = format!("Downloading: {} → {}/", title, dl.path)
            }
            Err(e) => {
                self.status = format!("Download failed: {}", e)
            }
        }
    }
}
