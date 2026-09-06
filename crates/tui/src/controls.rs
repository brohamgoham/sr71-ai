use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::{App, Overlay, Playback, Tab},
    observer::ObserveError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Control {
    Continue,
    Quit,
}

pub fn key(app: &mut App, key: KeyEvent) -> Result<Control, ObserveError> {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(Control::Quit);
    }
    if app.overlay == Overlay::Search {
        search(app, key.code);
        return Ok(Control::Continue);
    }
    if key.code == KeyCode::Char('q') {
        return Ok(Control::Quit);
    }
    if key.code == KeyCode::Esc {
        if app.overlay != Overlay::None {
            app.overlay = Overlay::None;
        } else if !app.query.is_empty() {
            app.query.clear();
        } else {
            app.following = None;
        }
        return Ok(Control::Continue);
    }
    if app.overlay != Overlay::None {
        return Ok(Control::Continue);
    }
    navigation(app, key.code)?;
    Ok(Control::Continue)
}

fn navigation(app: &mut App, key: KeyCode) -> Result<(), ObserveError> {
    if key == KeyCode::Char('1') {
        app.tab = Tab::Watch;
    }
    if key == KeyCode::Char('2') {
        app.tab = Tab::Cast;
    }
    if key == KeyCode::Char('3') {
        app.tab = Tab::Economy;
    }
    if matches!(key, KeyCode::Down | KeyCode::Char('j')) {
        app.move_selection(true);
    }
    if matches!(key, KeyCode::Up | KeyCode::Char('k')) {
        app.move_selection(false);
    }
    if key == KeyCode::Char('f') {
        app.follow();
    }
    if key == KeyCode::Enter {
        app.overlay = Overlay::Inspect;
    }
    if key == KeyCode::Char('?') {
        app.overlay = Overlay::Help;
    }
    if key == KeyCode::Char('/') {
        app.overlay = Overlay::Search;
        app.feed_offset = 0;
    }
    if key == KeyCode::Char(' ') {
        app.toggle_pause();
    }
    if key == KeyCode::Char('g') {
        app.go_live()?;
    }
    if key == KeyCode::Char('r') {
        app.replay()?;
    }
    if key == KeyCode::Char('s') {
        app.speed = app.speed.next();
    }
    if key == KeyCode::Char(']') {
        app.next_moment(true)?;
    }
    if key == KeyCode::Char('[') {
        app.next_moment(false)?;
    }
    if key == KeyCode::Right {
        app.step(true)?;
    }
    if key == KeyCode::Left {
        app.step(false)?;
    }
    if key == KeyCode::PageUp {
        app.playback = Playback::Paused;
        app.feed_offset = (app.feed_offset + 1).min(app.feed().len().saturating_sub(1));
    }
    if key == KeyCode::PageDown {
        app.playback = Playback::Paused;
        app.feed_offset = app.feed_offset.saturating_sub(1);
    }
    Ok(())
}

fn search(app: &mut App, key: KeyCode) {
    if key == KeyCode::Esc {
        app.query.clear();
        app.overlay = Overlay::None;
    }
    if key == KeyCode::Enter {
        app.overlay = Overlay::None;
    }
    if key == KeyCode::Backspace {
        app.query.pop();
    }
    if let KeyCode::Char(ch) = key
        && app.query.len() < 256
        && !ch.is_control()
    {
        app.query.push(ch);
    }
}
