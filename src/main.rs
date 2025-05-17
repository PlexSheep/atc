use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    prelude::{Constraint, Layout, Margin},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};

mod error;
mod level;
mod plane;
mod world;

use level::Level;
use tracing::{info, trace};

#[derive(Debug)]
pub struct App {
    state: GameState,
    level: Level,
    status_info: Option<String>,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
enum GameState {
    #[default]
    Startup,
    Ongoing,
    Results,
    Exit,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            level: Level::builtin(),
            status_info: Default::default(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.state != GameState::Exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
            match self.state {
                GameState::Startup => {
                    self.state = GameState::Ongoing;
                }
                GameState::Ongoing => match self.level.tick() {
                    world::State::Onging => (),
                    other => {
                        self.status_info = Some(format!("{other}"));
                        self.state = GameState::Results;
                    }
                },
                GameState::Results => self.state = GameState::Exit,
                GameState::Exit => break,
            }
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let whole_area = frame.area().inner(Margin::default());
        let chunks =
            Layout::vertical([Constraint::Length(frame.area().height), Constraint::Min(3)])
                .split(whole_area);
        let map_area = chunks[0];
        let status_area = chunks[1];

        let title = Line::from("Air Traffic Controller")
            .bold()
            .blue()
            .centered();
        let map: String = self.level.render();
        frame.render_widget(
            Paragraph::new(map).block(Block::bordered().title(title)),
            map_area,
        );
        if let Some(status_info) = self.status_info.take() {
            frame.render_widget(
                Paragraph::new(status_info).block(Block::bordered()),
                status_area,
            )
        }
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.state = GameState::Exit;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn setup_logging() {
    let logfile = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("/tmp/atc.log")
        .unwrap();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(
            #[cfg(debug_assertions)]
            tracing::Level::TRACE,
            #[cfg(not(debug_assertions))]
            tracing::Level::INFO,
        )
        .without_time()
        .with_file(false)
        .with_writer(logfile)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("could not setup logger");
    trace!("Setup logging");
}

fn main() -> color_eyre::Result<()> {
    setup_logging();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
