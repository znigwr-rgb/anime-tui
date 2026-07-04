use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::{App, InputMode};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    render_search_bar(frame, chunks[0], app);
    render_results(frame, chunks[1], app);
    render_status_bar(frame, chunks[2], app);
}

fn render_search_bar(frame: &mut Frame, area: Rect, app: &App) {
    let title = if app.loading {
        " Searching... "
    } else if app.spanish_filter {
        " Search [Español ON] "
    } else {
        " Search "
    };

    let border_color = if app.spanish_filter {
        Color::Yellow
    } else if app.input_mode == InputMode::Searching {
        Color::Cyan
    } else {
        Color::White
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);

    if app.input_mode == InputMode::Searching {
        let cursor_pos = app.query.len() as u16;
        let query_text = if app.query.is_empty() {
            "Type anime name and press Enter..."
        } else {
            &app.query
        };
        let paragraph = Paragraph::new(query_text).block(block);
        frame.render_widget(paragraph, area);
        frame.set_cursor_position((inner.x + cursor_pos, inner.y));
    } else {
        let paragraph = Paragraph::new(app.query.as_str()).block(block);
        frame.render_widget(paragraph, area);
    }
}

fn render_results(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Results ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    if app.results.is_empty() {
        let text = if app.loading {
            vec![Line::from(Span::styled(
                "Searching...",
                Style::default().fg(Color::Yellow),
            ))]
        } else {
            vec![
                Line::from(Span::styled(
                    "No results yet",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "Type a query above and press Enter",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        };
        let paragraph = Paragraph::new(Text::from(text))
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let widths = [
        Constraint::Percentage(55),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(12),
    ];

    let header_cells = ["Name", "Size", "Seeders", "Leech", "Source"]
        .iter()
        .map(|h| Cell::from(Line::from(Span::styled(*h, Style::default().add_modifier(Modifier::BOLD)))));
    let header = Row::new(header_cells)
        .style(Style::default().fg(Color::Cyan))
        .height(1);

    let visible_results: Vec<&crate::sources::TorrentEntry> = app
        .results
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .collect();

    let rows: Vec<Row> = visible_results
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let idx = app.scroll_offset + i;
            let is_selected = idx == app.selected;

            let cells = vec![
                Cell::from(entry.title.as_str()),
                Cell::from(entry.fmt_size()),
                Cell::from(entry.seeders.to_string()),
                Cell::from(entry.leechers.to_string()),
                Cell::from(entry.fmt_source()),
            ];

            if is_selected {
                Row::new(cells).style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else if idx % 2 == 0 {
                Row::new(cells).style(Style::default().bg(Color::Rgb(10, 10, 10)))
            } else {
                Row::new(cells)
            }
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(block);

    frame.render_widget(table, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let text = if app.input_mode == InputMode::Searching {
        "Type and press Enter to search | ↑↓ navigate | d download | c copy | s Spanish | q quit"
    } else {
        &app.status
    };

    let paragraph = Paragraph::new(Span::styled(
        text,
        Style::default().fg(Color::DarkGray),
    ))
    .block(block);

    frame.render_widget(paragraph, area);
}
