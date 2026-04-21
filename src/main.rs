mod app;
mod definition;
mod event;
mod keyboard;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::Event;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::ExecutableCommand;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::App;

fn main() -> Result<()> {
    // Parse CLI args
    let args: Vec<String> = std::env::args().collect();
    let definition_path = parse_definition_arg(&args);

    // Set up logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_writer(io::stderr)
        .init();

    // Set up terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, definition_path);

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    definition_path: Option<PathBuf>,
) -> Result<()> {
    let mut app = App::new(definition_path);
    app.scan();

    loop {
        terminal.draw(|frame| app.draw(frame))?;

        if app.should_quit {
            return Ok(());
        }

        if let Some(event) = event::poll_event(Duration::from_millis(100))?
            && !matches!(event, Event::Resize(_, _)) {
                app.handle_event(event);
            }
    }
}

fn parse_definition_arg(args: &[String]) -> Option<PathBuf> {
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--definition" | "-d" => {
                return iter.next().map(PathBuf::from);
            }
            _ if arg.starts_with("--definition=") => {
                return Some(PathBuf::from(arg.strip_prefix("--definition=").unwrap()));
            }
            _ => {}
        }
    }
    None
}
