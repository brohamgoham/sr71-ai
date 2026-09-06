use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Modifier,
    text::Line,
    widgets::{Cell, Paragraph, Row, Sparkline, Table},
};

use super::{CYAN, GOLD, MUTED, Motion, PAPER, Theme, safe, wrapped};
use crate::{
    app::App,
    model::{DecisionNote, LifeState, ObserverInput},
};

pub fn cast_list(frame: &mut Frame<'_>, app: &App, theme: Theme, area: Rect) {
    let height = usize::from(area.height.saturating_sub(3));
    let selected = usize::from(app.selected.0);
    let start = selected
        .saturating_sub(height / 2)
        .min(app.world.agents.len().saturating_sub(height));
    let rows: Vec<Row<'static>> = app
        .world
        .agents
        .iter()
        .skip(start)
        .take(height)
        .map(|agent| {
            let selected = agent.resident.id == app.selected;
            let prefix = if selected { ">" } else { " " };
            let style = if selected {
                theme.ink(CYAN).add_modifier(Modifier::BOLD)
            } else {
                theme.ink(PAPER)
            };
            Row::new([
                format!("{prefix} {}", agent.resident.name),
                agent.balance.to_string(),
                if agent.activity == crate::model::Activity::Deciding
                    && theme.motion == Motion::Subtle
                {
                    format!("deciding{}", ".".repeat(app.playhead as usize % 3))
                } else {
                    agent.activity.label().to_owned()
                },
            ])
            .style(style)
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Min(11),
            Constraint::Length(7),
            Constraint::Length(9),
        ],
    )
    .header(Row::new(["RESIDENT", "SUI", "ACTIVITY"]).style(theme.ink(MUTED)))
    .block(theme.block(format!(" THE CAST · {} residents ", app.world.agents.len())));
    frame.render_widget(table, area);
}

pub fn resident(frame: &mut Frame<'_>, app: &App, theme: Theme, area: Rect) {
    let Some(agent) = app.world.agent(app.selected) else {
        return;
    };
    let title = format!(
        " {}{} ",
        agent.resident.name.to_uppercase(),
        if app.following == Some(app.selected) {
            " · FOLLOWING"
        } else {
            ""
        }
    );
    let block = theme.block(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let life = match agent.resident.life {
        LifeState::Alive => "alive",
    };
    let mut lines = vec![
        Line::from(format!("born {} · {life}", safe(&agent.resident.origin)))
            .style(theme.ink(MUTED)),
        Line::from(format!(
            "{} SUI · now {}",
            agent.balance,
            app.world.bracket(agent.balance)
        ))
        .style(theme.ink(GOLD).add_modifier(Modifier::BOLD)),
        Line::from(format!(
            "gave {} / received {}",
            agent.given, agent.received
        ))
        .style(theme.ink(MUTED)),
        Line::from(""),
    ];
    match &agent.note {
        Some(DecisionNote::Visible { text }) => {
            lines.push(Line::from("DECISION NOTE · SELF-REPORT").style(theme.ink(CYAN)));
            for line in wrapped(text, usize::from(inner.width)) {
                lines.push(Line::from(line).style(theme.ink(PAPER)));
            }
        }
        Some(DecisionNote::Sealed { record_id }) => {
            lines.push(Line::from("SEALED NOTE · not available").style(theme.ink(MUTED)));
            lines.push(Line::from(safe(record_id)).style(theme.ink(MUTED)));
        }
        None => {
            lines.push(Line::from("No decision note available yet.").style(theme.ink(MUTED)));
            for line in wrapped(&agent.resident.disposition, usize::from(inner.width)) {
                lines.push(Line::from(line).style(theme.ink(MUTED)));
            }
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn cast(frame: &mut Frame<'_>, app: &App, theme: Theme, area: Rect) {
    let rows = Layout::vertical([Constraint::Min(8), Constraint::Length(8)])
        .spacing(1)
        .split(area);
    let available = usize::from(rows[0].height.saturating_sub(3));
    let start = usize::from(app.selected.0)
        .saturating_sub(available / 2)
        .min(app.world.agents.len().saturating_sub(available));
    let table_rows: Vec<Row<'static>> = app
        .world
        .agents
        .iter()
        .skip(start)
        .take(available)
        .map(|a| {
            let style = if a.resident.id == app.selected {
                theme.ink(CYAN).add_modifier(Modifier::BOLD)
            } else {
                theme.ink(PAPER)
            };
            let prefix = if a.resident.id == app.selected {
                "> "
            } else {
                "  "
            };
            Row::new([
                Cell::from(format!("{prefix}{}", a.resident.name)),
                Cell::from(safe(&a.resident.origin)),
                Cell::from(app.world.bracket(a.balance).to_owned()),
                Cell::from(a.balance.to_string()),
                Cell::from(
                    a.posts
                        .get(&app.world.epoch)
                        .copied()
                        .unwrap_or(0)
                        .to_string(),
                ),
                Cell::from(a.activity.label()),
            ])
            .style(style)
        })
        .collect();
    let table = Table::new(
        table_rows,
        [
            Constraint::Min(12),
            Constraint::Length(13),
            Constraint::Length(13),
            Constraint::Length(9),
            Constraint::Length(5),
            Constraint::Length(9),
        ],
    )
    .header(
        Row::new(["RESIDENT", "BORN", "NOW", "SUI", "POSTS", "ACTIVITY"]).style(theme.ink(MUTED)),
    )
    .block(theme.block(" THE CAST · stable identity, changing wealth ".into()));
    frame.render_widget(table, rows[0]);
    let detail = Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)])
        .spacing(1)
        .split(rows[1]);
    resident(frame, app, theme, detail[0]);
    rates(frame, app, theme, detail[1]);
}

fn rates(frame: &mut Frame<'_>, app: &App, theme: Theme, area: Rect) {
    let Some(agent) = app.world.agent(app.selected) else {
        return;
    };
    let block = theme.block(" POSTS / EPOCH · observed, not inferred ".into());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let rows = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(inner);
    let data: Vec<u64> = (1..=app.world.epoch)
        .map(|ep| agent.posts.get(&ep).copied().unwrap_or(0))
        .collect();
    frame.render_widget(
        Sparkline::default().data(&data).style(theme.ink(CYAN)),
        rows[0],
    );
    let previous: u64 = agent
        .posts
        .range(..app.world.epoch)
        .map(|(_, count)| count)
        .sum();
    let baseline = if app.world.epoch <= 1 {
        "N/A".to_owned()
    } else {
        format!("{:.1}", previous as f64 / f64::from(app.world.epoch - 1))
    };
    let current = agent.posts.get(&app.world.epoch).copied().unwrap_or(0);
    frame.render_widget(
        Paragraph::new(format!(
            "This epoch: {current}\nPrior average: {baseline} posts/epoch"
        ))
        .style(theme.ink(MUTED)),
        rows[1],
    );
}

pub fn economy(frame: &mut Frame<'_>, app: &App, theme: Theme, area: Rect) {
    let rows = Layout::vertical([Constraint::Length(7), Constraint::Min(6)])
        .spacing(1)
        .split(area);
    let (total, gini, whale) = app.world.metrics();
    let whale = whale.map_or("N/A".to_owned(), |v| format!("{:.1}%", v * 100.0));
    let sum = format!("{}.{:09}", total / 1_000_000_000, total % 1_000_000_000);
    let lines = vec![
        Line::from(format!(
            "  TOTAL WEALTH   {} SUI",
            sum.trim_end_matches('0').trim_end_matches('.')
        ))
        .style(theme.ink(GOLD).add_modifier(Modifier::BOLD)),
        Line::from(format!(
            "  {} gifts    Gini {gini:.3}    largest wallet {whale}",
            app.world.gifts
        )),
        Line::from(format!(
            "  Admin floor support +{} SUI · initial funding excluded",
            app.world.topups
        )),
        Line::from("  An open economy: gifts redistribute; top-ups add wealth.")
            .style(theme.ink(MUTED)),
        Line::from("  DEMO accounting only · no gas spent or on-chain activity")
            .style(theme.ink(MUTED)),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(theme.block(" THE ECONOMY ".into())),
        rows[0],
    );
    let columns = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
        .spacing(1)
        .split(rows[1]);
    let mut pairs: Vec<_> = app.world.pairs.iter().collect();
    pairs.sort_by_key(|a| std::cmp::Reverse(a.1.0));
    let transfers: Vec<Line<'static>> = pairs
        .iter()
        .map(|((from, to), (amount, count))| {
            Line::from(format!(
                " {} → {}  {amount} SUI / {count} gifts",
                app.world.name(*from),
                app.world.name(*to)
            ))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(transfers)
            .block(theme.block(" WHO GAVE TO WHOM ".into()))
            .wrap(ratatui::widgets::Wrap { trim: false }),
        columns[0],
    );
    let moments: Vec<Line<'static>> = app
        .world
        .moments
        .iter()
        .rev()
        .map(|m| {
            Line::from(format!(
                " {}\n source {}",
                safe(&m.title),
                safe(&m.evidence.0)
            ))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(moments)
            .block(theme.block(" MOMENTS · source-backed ".into()))
            .wrap(ratatui::widgets::Wrap { trim: false }),
        columns[1],
    );
}

pub fn inspection(app: &App) -> Vec<Line<'static>> {
    let Some(agent) = app.world.agent(app.selected) else {
        return Vec::new();
    };
    let mut lines = vec![
        Line::from(format!(
            "{} · born {} · now {}",
            agent.resident.name,
            safe(&agent.resident.origin),
            app.world.bracket(agent.balance)
        )),
        Line::from(safe(&agent.resident.disposition)),
        Line::from(""),
        Line::from("PUBLIC EVIDENCE · newest first · fixture provenance"),
    ];
    for event in app
        .world
        .events
        .iter()
        .rev()
        .filter(|e| app.world.involves(e, app.selected))
        .take(6)
    {
        let description = match &event.input {
            ObserverInput::Posted {
                text,
                reply_to,
                transaction,
                ..
            } => {
                format!(
                    "Post: {}\n  tx {transaction} / reply {}",
                    safe(text),
                    reply_to.as_ref().map_or("none", |id| id.0.as_str())
                )
            }
            ObserverInput::Gave {
                from,
                to,
                amount,
                transaction,
            } => {
                format!(
                    "Gift {} → {}: {amount} SUI\n  tx {transaction}",
                    app.world.name(*from),
                    app.world.name(*to)
                )
            }
            ObserverInput::TopUp {
                amount,
                transaction,
                ..
            } => format!("Admin top-up {amount} SUI / {transaction}"),
            ObserverInput::Note { note, .. } => match note {
                DecisionNote::Visible { text } => format!("Self-report: {}", safe(text)),
                DecisionNote::Sealed { record_id } => {
                    format!("Sealed: {} (no plaintext available)", safe(record_id))
                }
            },
            ObserverInput::Activity { state, .. } => format!("Runtime status: {}", state.label()),
            ObserverInput::Epoch { .. } | ObserverInput::Connection { .. } => String::new(),
        };
        lines.push(Line::from(safe(&format!(
            "{}  {:02}:{:02}  {description}",
            event.id.0,
            event.at / 60,
            event.at % 60
        ))));
        lines.push(Line::from(""));
    }
    lines
}
