**ANIME-TUI**

---
- Anime torrent finder and downloader from the terminal. Key features:
- Search Nyaa.si via RSS (and SubsPlease as a backup) 
- Toggleable Spanish filter (latino/castellano dubs)
- Magnet copy to clipboard, opens with external client as fallback 
- Lightweight TUI with Ratatui + Crossterm Written in Rust, no JS dependencies, no Cloudflare.

---
NAVEGATION:
| Key | Action |
|-----|--------|
| **Search mode** (cursor active) | |
| `any key` | Type query |
| `Enter` | Execute search |
| `Backspace` | Delete last character |
| `Esc` | Exit search mode |
| **Results mode** | |
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` / `/` | Enter search mode |
| `d` | Download with aria2c |
| `c` | Copy magnet to clipboard |
| `s` | Toggle Spanish filter |
| `q` / `Esc` | Quit |
