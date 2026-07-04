mod app;
mod download;
mod sources;
mod ui;

use app::{App, InputMode};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io::{self, Write};
use std::process::{Command, Stdio};

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.input_mode {
                InputMode::Searching => match key.code {
                    KeyCode::Enter => {
                        app.input_mode = InputMode::Normal;
                        app.search().await;
                    }
                    KeyCode::Char(c) => {
                        app.push_char(c);
                    }
                    KeyCode::Backspace => {
                        app.pop_char();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(());
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next();
                    }
                    KeyCode::Enter | KeyCode::Char('/') => {
                        app.input_mode = InputMode::Searching;
                    }
                    KeyCode::Char('d') => {
                        app.download_selected();
                    }
                    KeyCode::Char('c') => {
                        if let Some(magnet) = app.get_selected_magnet() {
                            copy_to_clipboard(&magnet);
                            app.status = "Magnet link copied to clipboard".into();
                        }
                    }
                    KeyCode::Char('s') => {
                        app.toggle_spanish();
                    }
                    _ => {}
                },
            }
        }
    }
}

fn copy_to_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}
