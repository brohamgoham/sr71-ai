use std::{fs, path::PathBuf, time::Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend, style::Color, widgets::BorderType};
use tui::{
    app::{App, Overlay, Playback, Tab},
    controls::{self, Control},
    fixture,
    model::{AgentId, DecisionNote},
    ui::{self, ColorMode, Motion, Theme},
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn press(app: &mut App, code: KeyCode) -> TestResult {
    assert_eq!(
        controls::key(app, KeyEvent::new(code, KeyModifiers::NONE))?,
        Control::Continue
    );
    Ok(())
}

fn screen(app: &App, size: (u16, u16), theme: Theme) -> Result<String, Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(TestBackend::new(size.0, size.1))?;
    terminal.draw(|frame| ui::render(frame, app, theme))?;
    let buffer = terminal.backend().buffer();
    let mut output = String::new();
    for y in 0..size.1 {
        let mut line = String::new();
        for x in 0..size.0 {
            line.push_str(buffer[(x, y)].symbol());
        }
        output.push_str(line.trim_end());
        output.push('\n');
    }
    Ok(output)
}

#[test]
fn keyboard_flow_filters_follows_and_seeks_without_mutating_recording() -> TestResult {
    let mut app = App::new(fixture::story()?)?;
    app.seek(167.0)?;
    press(&mut app, KeyCode::Char('2'))?;
    assert_eq!(app.tab, Tab::Cast);
    press(&mut app, KeyCode::Down)?;
    assert_eq!(app.selected, AgentId(15));
    press(&mut app, KeyCode::Char('f'))?;
    assert_eq!(app.following, Some(AgentId(15)));
    assert!(
        app.feed()
            .iter()
            .all(|e| app.world.involves(e, AgentId(15)))
    );
    press(&mut app, KeyCode::Char('/'))?;
    for ch in "Thank".chars() {
        press(&mut app, KeyCode::Char(ch))?;
    }
    press(&mut app, KeyCode::Enter)?;
    assert_eq!(app.feed().len(), 1);
    press(&mut app, KeyCode::Enter)?;
    assert_eq!(app.overlay, Overlay::Inspect);
    press(&mut app, KeyCode::Esc)?;
    press(&mut app, KeyCode::Char('3'))?;
    assert_eq!(app.tab, Tab::Economy);
    press(&mut app, KeyCode::Char('?'))?;
    assert_eq!(app.overlay, Overlay::Help);
    press(&mut app, KeyCode::Esc)?;
    press(&mut app, KeyCode::Char('r'))?;
    assert_eq!(app.playback, Playback::Replay);
    assert_eq!(app.world.gifts, 0);
    press(&mut app, KeyCode::Char(']'))?;
    assert_eq!(app.world.gifts, 1);
    press(&mut app, KeyCode::Char(' '))?;
    assert_eq!(app.playback, Playback::Replay);
    assert_eq!(
        controls::key(
            &mut app,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)
        )?,
        Control::Quit
    );
    Ok(())
}

#[test]
fn search_capture_scroll_and_resize_keep_selection() -> TestResult {
    let mut app = App::new(fixture::story()?)?;
    app.seek(167.0)?;
    press(&mut app, KeyCode::PageUp)?;
    assert_eq!(app.feed_offset, 1);
    assert_eq!(app.playback, Playback::Paused);
    press(&mut app, KeyCode::PageDown)?;
    assert_eq!(app.feed_offset, 0);
    press(&mut app, KeyCode::Char('/'))?;
    press(&mut app, KeyCode::Char('q'))?;
    assert_eq!(app.query, "q");
    press(&mut app, KeyCode::Backspace)?;
    assert_eq!(app.query, "");
    press(&mut app, KeyCode::Esc)?;
    let selected = app.selected;
    for size in [(120, 40), (100, 30), (80, 24), (59, 14), (1, 1)] {
        assert!(!screen(&app, size, Theme::default())?.is_empty());
        assert_eq!(app.selected, selected);
    }
    assert!(screen(&app, (59, 14), Theme::default())?.contains("Resize"));
    Ok(())
}

#[test]
fn sealed_notes_never_render_plaintext_and_control_codes_are_harmless() -> TestResult {
    let mut app = App::new(fixture::story()?)?;
    app.selected = AgentId(11);
    app.seek(231.0)?;
    assert!(matches!(
        app.world.agents[11].note,
        Some(DecisionNote::Sealed { .. })
    ));
    let view = screen(&app, (120, 40), Theme::default())?;
    assert!(view.contains("SEALED NOTE"));
    assert!(view.contains("demo-sealed-001"));
    let hostile = "\u{1b}]52;c;secret\u{7}\u{202e}pretend\u{8}";
    assert!(!ui::safe(hostile).contains('\u{1b}'));
    assert!(!ui::safe(hostile).contains('\u{7}'));
    app.world.agents[11].note = Some(DecisionNote::Visible {
        text: hostile.into(),
    });
    app.overlay = Overlay::Inspect;
    let view = screen(&app, (120, 40), Theme::default())?;
    assert!(!view.contains('\u{1b}'));
    assert!(!view.contains('\u{202e}'));
    assert_eq!(
        ui::wrapped("e\u{301} e\u{301}", 1),
        vec!["e\u{301}", "e\u{301}"]
    );
    Ok(())
}

#[test]
fn quiet_and_failure_scenarios_have_honest_status() -> TestResult {
    let quiet = fixture::parse(include_str!("../fixtures/quiet.json"))?;
    let mut app = App::new(quiet)?;
    app.seek(599.0)?;
    assert!(screen(&app, (120, 40), Theme::default())?.contains("There is room for silence"));
    assert_eq!(app.world.agents.len(), 16);
    let mut app = App::new(fixture::parse(include_str!("../fixtures/reconnect.json"))?)?;
    app.seek(190.0)?;
    assert!(screen(&app, (120, 40), Theme::default())?.contains("SIGNAL LOST"));
    app.seek(241.0)?;
    assert!(screen(&app, (120, 40), Theme::default())?.contains("signal current"));
    Ok(())
}

#[test]
fn no_color_ascii_borders_and_reduced_motion_are_respected() -> TestResult {
    let mut app = App::new(fixture::story()?)?;
    app.seek(167.0)?;
    let theme = Theme {
        color: ColorMode::Plain,
        border: BorderType::Plain,
        motion: Motion::Reduced,
    };
    let mut terminal = Terminal::new(TestBackend::new(120, 40))?;
    terminal.draw(|f| ui::render(f, &app, theme))?;
    assert!(
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .all(|cell| cell.fg == Color::Reset && cell.bg == Color::Reset)
    );
    let view = screen(&app, (120, 40), theme)?;
    assert!(view.contains("+ THE PUBLIC SQUARE"));
    assert!(!view.contains('╭'));
    Ok(())
}

#[test]
fn terminal_snapshots_cover_three_sizes_and_all_views() -> TestResult {
    let mut app = App::new(fixture::story()?)?;
    app.seek(167.0)?;
    for (tab, name, size) in [
        (Tab::Watch, "watch-120x40", (120, 40)),
        (Tab::Watch, "watch-100x30", (100, 30)),
        (Tab::Watch, "watch-80x24", (80, 24)),
        (Tab::Cast, "cast-120x40", (120, 40)),
        (Tab::Economy, "economy-120x40", (120, 40)),
    ] {
        app.tab = tab;
        let actual = screen(&app, size, Theme::default())?;
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots");
        let path = directory.join(format!("{name}.txt"));
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            fs::create_dir_all(&directory)?;
            fs::write(&path, &actual)?;
        }
        assert_eq!(actual, fs::read_to_string(path)?, "snapshot {name}");
    }
    Ok(())
}

#[test]
fn burst_render_and_input_remain_responsive() -> TestResult {
    let mut app = App::new(fixture::parse(include_str!("../fixtures/burst.json"))?)?;
    app.seek(20.0)?;
    let before = Instant::now();
    let view = screen(&app, (120, 40), Theme::default())?;
    press(&mut app, KeyCode::Char('2'))?;
    assert!(view.contains("Public post 1000"));
    assert_eq!(app.tab, Tab::Cast);
    assert_eq!(app.feed().len(), 1000);
    assert!(
        before.elapsed().as_millis() < 500,
        "burst render must remain interactive even in debug"
    );
    Ok(())
}
