use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
};

use super::{CYAN, GOLD, MUTED, PAPER, Theme, panels, safe, wrapped};
use crate::{
    app::App,
    model::{Envelope, ObserverInput},
};

pub fn render(frame: &mut Frame<'_>, app: &App, theme: Theme, area: Rect) {
    if area.width < 95 {
        feed(frame, app, theme, area);
        return;
    }
    let columns = Layout::horizontal([Constraint::Min(48), Constraint::Length(36)])
        .spacing(1)
        .split(area);
    feed(frame, app, theme, columns[0]);
    let sidebar = Layout::vertical([
        Constraint::Length(11),
        Constraint::Min(6),
        Constraint::Length(6),
    ])
    .spacing(1)
    .split(columns[1]);
    panels::cast_list(frame, app, theme, sidebar[0]);
    panels::resident(frame, app, theme, sidebar[1]);
    let lines = app
        .world
        .moments
        .iter()
        .rev()
        .take(3)
        .map(|moment| {
            Line::from(format!(
                "{:02}:{:02} {}",
                moment.at / 60,
                moment.at % 60,
                safe(&moment.title)
            ))
            .style(theme.ink(MUTED))
        })
        .collect::<Vec<_>>();
    let lines = if lines.is_empty() {
        vec![Line::from("The story is still beginning.")]
    } else {
        lines
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(theme.block(" MOMENTS · [ ] to revisit ".into()))
            .wrap(ratatui::widgets::Wrap { trim: false }),
        sidebar[2],
    );
}

fn feed(frame: &mut Frame<'_>, app: &App, theme: Theme, area: Rect) {
    let title = if let Some(id) = app.following {
        format!(
            " FOLLOWING {} · f to release ",
            app.world.name(id).to_uppercase()
        )
    } else {
        " THE PUBLIC SQUARE ".to_owned()
    };
    let block = theme.block(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 4 {
        return;
    }
    let feed = app.feed();
    if feed.is_empty() {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from("  There is room for silence.").style(theme.ink(GOLD)),
                Line::from(""),
                Line::from("  No public actions match this view.").style(theme.ink(MUTED)),
                Line::from("  Residents remain here. No one is removed.").style(theme.ink(MUTED)),
                Line::from("  Try ] for a moment, / to filter, or g to catch up.")
                    .style(theme.ink(MUTED)),
            ]),
            inner,
        );
        return;
    }
    let count = feed.len().saturating_sub(app.feed_offset);
    let mut cards: Vec<Vec<Line<'static>>> = Vec::new();
    for (index, event) in feed.iter().take(count).enumerate() {
        if let ObserverInput::Posted { transaction, .. } = &event.input
            && index > 0
            && matches!(&feed[index - 1].input,
                ObserverInput::Gave { transaction: gift_tx, .. } if gift_tx == transaction)
        {
            continue;
        }
        let next = feed.get(index + 1).copied().filter(|_| index + 1 < count);
        cards.push(card(
            app,
            theme,
            event,
            next,
            usize::from(inner.width.saturating_sub(2)),
        ));
    }
    let available = usize::from(inner.height);
    let mut lines = Vec::new();
    for card in cards.iter().rev() {
        if !lines.is_empty() && lines.len() + card.len() > available {
            break;
        }
        let mut older = card.clone();
        older.append(&mut lines);
        lines = older;
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn card(
    app: &App,
    theme: Theme,
    event: &Envelope,
    next: Option<&Envelope>,
    width: usize,
) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from("")];
    let time = format!("{:02}:{:02}", event.at / 60, event.at % 60);
    match &event.input {
        ObserverInput::Posted {
            agent,
            text,
            reply_to,
            to,
            ..
        } => {
            let addressee = to
                .map(|id| format!(" → {}", app.world.name(id)))
                .unwrap_or_default();
            let reply = reply_to
                .as_ref()
                .map(|id| format!("  ↳ {}", safe(&id.0)))
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {}{addressee}", app.world.name(*agent)),
                    theme.ink(CYAN).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("   {time}{reply}"), theme.ink(MUTED)),
            ]));
            for line in wrapped(text, width.saturating_sub(2)) {
                lines.push(Line::from(format!(" {line}")).style(theme.ink(PAPER)));
            }
            lines.push(
                Line::from(format!(" {} · recorded demo post", safe(&event.id.0)))
                    .style(theme.ink(MUTED)),
            );
        }
        ObserverInput::Gave {
            from,
            to,
            amount,
            transaction,
        } => {
            lines.push(
                Line::from(format!(
                    " {} → {}   {time}",
                    app.world.name(*from),
                    app.world.name(*to)
                ))
                .style(theme.ink(GOLD).add_modifier(Modifier::BOLD)),
            );
            lines.push(Line::from(format!(" GAVE {amount} SUI")).style(theme.ink(GOLD)));
            if let Some(Envelope {
                input:
                    ObserverInput::Posted {
                        text,
                        transaction: post_tx,
                        ..
                    },
                ..
            }) = next
                && post_tx == transaction
            {
                for line in wrapped(text, width.saturating_sub(2)) {
                    lines.push(Line::from(format!(" {line}")).style(theme.ink(PAPER)));
                }
            }
            lines.push(
                Line::from(format!(
                    " {} · completed in fixture · same transaction",
                    safe(&event.id.0)
                ))
                .style(theme.ink(MUTED)),
            );
        }
        ObserverInput::TopUp { agent, amount, .. } => {
            lines.push(
                Line::from(format!(" ADMIN → {}    {time}", app.world.name(*agent)))
                    .style(theme.ink(MUTED)),
            );
            lines.push(
                Line::from(format!(" Floor support +{amount} SUI · not an agent gift"))
                    .style(theme.ink(MUTED)),
            );
        }
        ObserverInput::Epoch { .. }
        | ObserverInput::Note { .. }
        | ObserverInput::Activity { .. }
        | ObserverInput::Connection { .. } => {}
    }
    lines
}
