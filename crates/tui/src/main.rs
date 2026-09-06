use std::{
    fs::{self, OpenOptions},
    io::{self, IsTerminal},
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{Terminal, backend::TestBackend, widgets::BorderType};
use tui::{
    app::App,
    controls::{self, Control},
    fixture,
    model::Fixture,
    ui::{self, ColorMode, Motion, Theme},
};

#[derive(Parser)]
#[command(
    name = "square",
    about = "A society to watch. Read-only ratatui prototype."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Play a labeled fictional recording. No keys, RPC or models required.
    Demo(Options),
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum Scenario {
    #[default]
    Story,
    Quiet,
    Burst,
    Reconnect,
}

#[derive(Args)]
struct Options {
    #[arg(long, value_enum, default_value_t = Scenario::Story)]
    scenario: Scenario,
    #[arg(long)]
    fixture: Option<PathBuf>,
    /// Render a plain terminal snapshot and exit (useful without a TTY).
    #[arg(long)]
    snapshot: Option<PathBuf>,
    #[arg(long, default_value_t = 120, value_parser = clap::value_parser!(u16).range(1..=500))]
    width: u16,
    #[arg(long, default_value_t = 40, value_parser = clap::value_parser!(u16).range(1..=200))]
    height: u16,
    #[arg(long, default_value_t = 0)]
    at: u64,
    #[arg(long)]
    no_color: bool,
    #[arg(long)]
    ascii: bool,
    #[arg(long)]
    reduced_motion: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let Command::Demo(options) = cli.command;
    logging()?;
    let fixture = load(&options)?;
    let mut app = App::new(fixture)?;
    if options.at > app.fixture.duration_seconds {
        bail!(
            "--at {} exceeds fixture duration {}; choose a time in the recording",
            options.at,
            app.fixture.duration_seconds
        );
    }
    app.live_head = options.at as f64;
    app.seek(options.at as f64)?;
    let theme = Theme {
        color: if options.no_color || std::env::var_os("NO_COLOR").is_some() {
            ColorMode::Plain
        } else {
            ColorMode::Color
        },
        border: if options.ascii {
            BorderType::Plain
        } else {
            BorderType::Rounded
        },
        motion: if options.reduced_motion {
            Motion::Reduced
        } else {
            Motion::Subtle
        },
    };
    if let Some(path) = &options.snapshot {
        return snapshot(&app, theme, &options, path);
    }
    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        bail!(
            "the viewer needs an interactive terminal; use demo --snapshot <file> for a headless preview"
        );
    }
    tracing::info!(run = %app.fixture.run_id, "starting fictional demo; no external services");
    let mut terminal = ratatui::try_init().context("cannot initialize terminal; check TERM")?;
    let _restore = Restore;
    run(&mut terminal, &mut app, theme)
}

fn load(options: &Options) -> Result<Fixture> {
    if let Some(path) = &options.fixture {
        return Ok(fixture::read(path)?);
    }
    let text = match options.scenario {
        Scenario::Story => include_str!("../fixtures/story.json"),
        Scenario::Quiet => include_str!("../fixtures/quiet.json"),
        Scenario::Burst => include_str!("../fixtures/burst.json"),
        Scenario::Reconnect => include_str!("../fixtures/reconnect.json"),
    };
    Ok(fixture::parse(text)?)
}

struct Restore;
impl Drop for Restore {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

fn logging() -> Result<()> {
    fs::create_dir_all(".local")
        .context("cannot create .local log directory; check write permissions")?;
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(".local/viewer.log")
        .context("cannot open .local/viewer.log; check write permissions")?;
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(log)
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .try_init()
        .map_err(|error| anyhow::anyhow!("cannot initialize file logging: {error}"))
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App, theme: Theme) -> Result<()> {
    let mut previous = Instant::now();
    loop {
        terminal
            .draw(|frame| ui::render(frame, app, theme))
            .context("cannot render viewer; check terminal connection")?;
        if event::poll(Duration::from_millis(100)).context("cannot poll terminal input")? {
            let input = event::read().context("cannot read terminal input")?;
            if let Event::Key(key) = input
                && key.kind == KeyEventKind::Press
                && controls::key(app, key)? == Control::Quit
            {
                break;
            }
        }
        let now = Instant::now();
        app.tick(now.duration_since(previous).as_secs_f64())?;
        previous = now;
    }
    tracing::info!("viewer exited; no simulation was stopped");
    Ok(())
}

fn snapshot(app: &App, theme: Theme, options: &Options, path: &PathBuf) -> Result<()> {
    let mut terminal = Terminal::new(TestBackend::new(options.width, options.height))?;
    terminal.draw(|frame| ui::render(frame, app, theme))?;
    let buffer = terminal.backend().buffer();
    let mut text = String::new();
    for y in 0..options.height {
        let mut line = String::new();
        for x in 0..options.width {
            line.push_str(buffer[(x, y)].symbol());
        }
        text.push_str(line.trim_end());
        text.push('\n');
    }
    fs::write(path, text).with_context(|| {
        format!(
            "cannot write snapshot {}; check destination",
            path.display()
        )
    })?;
    tracing::info!(path = %path.display(), "wrote terminal snapshot");
    Ok(())
}
