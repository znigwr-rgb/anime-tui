use scraper::{Html, Selector};

use super::TorrentEntry;

pub async fn search(query: &str, spanish: bool) -> Result<Vec<TorrentEntry>, String> {
    let category = if spanish { "1_4" } else { "0_0" };
    let full_query = if spanish && !query.is_empty() {
        format!("{} latino OR {} castellano OR {} spanish OR {} español", query, query, query, query)
    } else {
        query.to_string()
    };

    let url = format!(
        "https://nyaa.si/?page=rss&q={}&c={}&f=0",
        urlencoding(&full_query),
        category
    );

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let xml = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Nyaa: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Nyaa: {}", e))?;

    parse_nyaa_rss(&xml)
}

fn parse_nyaa_rss(xml: &str) -> Result<Vec<TorrentEntry>, String> {
    let document = Html::parse_fragment(xml);
    let item_sel = Selector::parse("item").map_err(|e| e.to_string())?;

    let mut entries = Vec::new();

    for item in document.select(&item_sel) {
        let inner = item.inner_html();

        let title = tag(&inner, "title");
        let info_hash = tag(&inner, "nyaa:infohash");
        let seeders = tag(&inner, "nyaa:seeders").parse().unwrap_or(0);
        let leechers = tag(&inner, "nyaa:leechers").parse().unwrap_or(0);
        let size = tag(&inner, "nyaa:size");

        if title.is_empty() || info_hash.is_empty() {
            continue;
        }

        let magnet = build_magnet(&info_hash, &title);

        entries.push(TorrentEntry {
            title,
            magnet,
            size,
            seeders,
            leechers,
            source: "Nyaa",
        });
    }

    Ok(entries)
}

fn tag(html: &str, name: &str) -> String {
    let re = regex_lite::Regex::new(&format!(
        r"<{}>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</{}>",
        regex_lite::escape(name),
        regex_lite::escape(name)
    ));

    let re = match re {
        Ok(r) => r,
        Err(_) => return String::new(),
    };

    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default()
}

fn build_magnet(info_hash: &str, name: &str) -> String {
    let trackers = [
        "http://nyaa.tracker.wf:7777/announce",
        "udp://open.stealth.si:80/announce",
        "udp://tracker.opentrackr.org:1337/announce",
        "udp://exodus.desync.com:6969/announce",
        "udp://tracker.torrent.eu.org:451/announce",
    ];

    let tr_params: Vec<String> = trackers
        .iter()
        .map(|t| format!("&tr={}", urlencoding(t)))
        .collect();

    format!(
        "magnet:?xt=urn:btih:{}&dn={}{}",
        info_hash,
        urlencoding(name),
        tr_params.concat()
    )
}

fn urlencoding(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss() {
        let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <item>
              <title>Death Note - Test</title>
              <nyaa:seeders>42</nyaa:seeders>
              <nyaa:leechers>7</nyaa:leechers>
              <nyaa:infoHash>abcdef1234567890abcdef1234567890abcdef12</nyaa:infoHash>
              <nyaa:size>1.2 GiB</nyaa:size>
            </item>
            <item>
              <title>Another Anime</title>
              <nyaa:seeders>10</nyaa:seeders>
              <nyaa:leechers>2</nyaa:leechers>
              <nyaa:infoHash>1234567890abcdef1234567890abcdef12345678</nyaa:infoHash>
              <nyaa:size>500 MiB</nyaa:size>
            </item>
          </channel>
        </rss>"#;

        let result = parse_nyaa_rss(rss).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Death Note - Test");
        assert_eq!(result[0].seeders, 42);
        assert_eq!(result[0].leechers, 7);
        assert_eq!(result[0].size, "1.2 GiB");
        assert!(result[0].magnet.starts_with("magnet:?xt=urn:btih:abcdef1234567890abcdef1234567890abcdef12"));
        assert_eq!(result[1].title, "Another Anime");
    }

    #[test]
    fn test_parse_rss_with_special_chars() {
        let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
            <item>
              <title>Test [1080p] x265</title>
              <nyaa:seeders>5</nyaa:seeders>
              <nyaa:leechers>1</nyaa:leechers>
              <nyaa:infoHash>deadbeefdeadbeefdeadbeefdeadbeefdeadbeef</nyaa:infoHash>
              <nyaa:size>2.5 GiB</nyaa:size>
            </item>
          </channel>
        </rss>"#;

        let result = parse_nyaa_rss(rss).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Test [1080p] x265");
    }

    #[test]
    fn test_parse_rss_empty() {
        let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
          <channel>
          </channel>
        </rss>"#;

        let result = parse_nyaa_rss(rss).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello%20world");
        assert_eq!(urlencoding("abc123"), "abc123");
        assert_eq!(urlencoding("a b"), "a%20b");
    }
}
