mod panels;
mod watch;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{App, Overlay, Playback, Tab},
    model::Connection,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorMode {
    #[default]
    Color,
    Plain,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Motion {
    #[default]
    Subtle,
    Reduced,
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub color: ColorMode,
    pub border: BorderType,
    pub motion: Motion,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            color: ColorMode::Color,
            border: BorderType::Rounded,
            motion: Motion::Subtle,
        }
    }
}

impl Theme {
    pub fn ink(self, color: Color) -> Style {
        if self.color == ColorMode::Plain {
            Style::default()
        } else {
            Style::default().fg(color)
        }
    }

    pub fn block(self, title: String) -> Block<'static> {
        let block = Block::default()
            .title(Line::from(title).style(self.ink(Color::Gray)))
            .borders(Borders::ALL)
            .border_type(self.border)
            .border_style(self.ink(Color::Rgb(50, 62, 76)));
        if self.border == BorderType::Plain {
            block.border_set(ratatui::symbols::border::Set {
                top_left: "+",
                top_right: "+",
                bottom_left: "+",
                bottom_right: "+",
                vertical_left: "|",
                vertical_right: "|",
                horizontal_top: "-",
                horizontal_bottom: "-",
            })
        } else {
            block
        }
    }
}

pub const GOLD: Color = Color::Rgb(237, 196, 112);
pub const CYAN: Color = Color::Rgb(107, 205, 196);
pub const MUTED: Color = Color::Rgb(145, 158, 177);
pub const PAPER: Color = Color::Rgb(226, 232, 241);

pub fn render(frame: &mut Frame<'_>, app: &App, theme: Theme) {
    let area = frame.area();
    let background = if theme.color == ColorMode::Color {
        Style::default().bg(Color::Rgb(15, 20, 29)).fg(PAPER)
    } else {
        Style::default()
    };
    frame.render_widget(Block::default().style(background), area);
    if area.width < 60 || area.height < 15 {
        frame.render_widget(
            Paragraph::new("THE SQUARE · DEMO\nResize to at least 60 × 15.\nq quits safely."),
            area,
        );
        return;
    }
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(7),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .margin(1)
    .split(area);
    header(frame, app, theme, rows[0]);
    match app.tab {
        Tab::Watch => watch::render(frame, app, theme, rows[1]),
        Tab::Cast => panels::cast(frame, app, theme, rows[1]),
        Tab::Economy => panels::economy(frame, app, theme, rows[1]),
    }
    status(frame, app, theme, rows[2]);
    frame.render_widget(
        Paragraph::new(
            "1/2/3 views  ↑↓ cast  f follow  ↵ inspect  space pause  r replay  ? keys  q quit",
        )
        .style(theme.ink(MUTED)),
        rows[3],
    );
    if app.overlay != Overlay::None {
        overlay(frame, app, theme);
    }
}

fn header(frame: &mut Frame<'_>, app: &App, theme: Theme, area: Rect) {
    let columns = Layout::horizontal([Constraint::Min(28), Constraint::Length(33)]).split(area);
    let title = Line::from(vec![
        Span::styled("THE SQUARE", theme.ink(GOLD).add_modifier(Modifier::BOLD)),
        Span::styled("  /  DEMO", theme.ink(CYAN)),
        Span::styled(format!("   epoch {:02}", app.world.epoch), theme.ink(MUTED)),
    ]);
    frame.render_widget(
        Paragraph::new(vec![
            title,
            Line::from(safe(&app.fixture.title)).style(theme.ink(MUTED)),
        ]),
        columns[0],
    );
    let tabs = [
        (Tab::Watch, "1 WATCH"),
        (Tab::Cast, "2 CAST"),
        (Tab::Economy, "3 ECONOMY"),
    ];
    let spans: Vec<_> = tabs
        .into_iter()
        .map(|(tab, title)| {
            let style = if app.tab == tab {
                theme.ink(CYAN).add_modifier(Modifier::BOLD)
            } else {
                theme.ink(MUTED)
            };
            Span::styled(format!(" {title} "), style)
        })
        .collect();
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(spans),
            Line::from("Fictional · no chain / API").style(theme.ink(MUTED)),
        ]),
        columns[1],
    );
}

fn status(frame: &mut Frame<'_>, app: &App, theme: Theme, area: Rect) {
    let mode = match app.playback {
        Playback::Live => "PLAYING DEMO",
        Playback::Paused => "VIEW PAUSED",
        Playback::Replay => "REPLAY",
    };
    let connection = match app.world.connection {
        Connection::Current => "signal current",
        Connection::Disconnected => "SIGNAL LOST · data stale",
        Connection::Recovering => "recovering history",
    };
    let end = if app.playhead >= app.fixture.duration_seconds as f64 {
        " · END OF DEMO"
    } else {
        ""
    };
    let at = app.playhead as u64;
    let ratio = app.playhead / app.fixture.duration_seconds as f64;
    let available = usize::from(area.width).saturating_sub(2);
    let filled = ((available as f64 * ratio) as usize).min(available);
    let timeline = format!("{}{}", "━".repeat(filled), "─".repeat(available - filled));
    let line = format!(
        "{mode}  {:02}:{:02} / {:02}:{:02}  {:.1}x  · {connection} · {} buffered{end}",
        at / 60,
        at % 60,
        app.fixture.duration_seconds / 60,
        app.fixture.duration_seconds % 60,
        app.speed.multiplier(),
        app.backlog()
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(timeline).style(theme.ink(CYAN)),
            Line::from(line).style(theme.ink(MUTED)),
        ]),
        area,
    );
}

fn overlay(frame: &mut Frame<'_>, app: &App, theme: Theme) {
    let area = frame.area();
    let width = area.width.saturating_sub(6).min(88);
    let height = area.height.saturating_sub(4).min(30);
    let popup = Rect::new(
        (area.width - width) / 2,
        (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);
    let (title, lines) = match app.overlay {
        Overlay::Help => (" THE VIEWER · esc closes ".to_owned(), help()),
        Overlay::Inspect => (
            " EVIDENCE & CHARACTER · esc closes ".to_owned(),
            panels::inspection(app),
        ),
        Overlay::Search => (
            " FILTER PUBLIC FEED · enter applies / esc clears ".to_owned(),
            vec![
                Line::from(format!("> {}", safe(&app.query))),
                Line::from(""),
                Line::from("Search names or public text. The recording keeps running."),
            ],
        ),
        Overlay::None => (String::new(), Vec::new()),
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(theme.block(title))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .style(theme.ink(PAPER)),
        popup,
    );
}

fn help() -> Vec<Line<'static>> {
    [
        "You are watching a fictional fixture, not autonomous agents.",
        "",
        "1 / 2 / 3      Watch / Cast / Economy",
        "Up / Down      Select a resident (selection is stable)",
        "f              Follow or unfollow the selected resident",
        "Enter          Inspect selected resident and source events",
        "Page Up/Down   Read older/newer feed cards (pauses view)",
        "Space          Pause / resume the presentation",
        "r              Replay from the start",
        "g              Return to the demo's live clock",
        "Left / Right   Step to previous / next event",
        "[ / ]          Previous / next recorded moment",
        "s              Replay speed: 0.5x / 1x / 2x / 4x",
        "/              Filter public feed by text or name",
        "Esc            Close overlay / clear filter / unfollow",
        "q / Ctrl-C     Quit the viewer",
        "",
        "Decision notes are self-reports, not verified motives.",
        "Sealed notes display only a record reference. No plaintext is loaded.",
        "Replay never changes the simulation. No wallet or API key is used.",
    ]
    .into_iter()
    .map(Line::from)
    .collect()
}

pub fn safe(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if (c.is_control() && c != '\n')
                || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
            {
                '�'
            } else {
                c
            }
        })
        .collect()
}

pub fn wrapped(input: &str, width: usize) -> Vec<String> {
    let clean = safe(input);
    let mut lines = Vec::new();
    for paragraph in clean.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if !line.is_empty() && line.width() + 1 + word.width() > width {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            for ch in word.chars() {
                if !line.is_empty()
                    && line.width() + unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)
                        > width
                {
                    lines.push(std::mem::take(&mut line));
                }
                line.push(ch);
            }
        }
        lines.push(line);
    }
    lines
}
