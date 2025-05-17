use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};

mod level;
mod plane;
mod world;

use level::Level;
use tracing::{info, trace};

fn main() -> color_eyre::Result<()> {
    setup_logging();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

#[derive(Debug)]
pub struct App {
    running: bool,
    pause: bool,
    must_exit: bool,
    level: Level,
    status_info: Option<String>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            pause: false,
            level: Level::builtin(),
            status_info: Default::default(),
            must_exit: false,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            self.running = !self.must_exit;
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
            if !self.pause {
                match self.level.tick() {
                    world::State::Onging => (),
                    other => {
                        self.status_info = Some(format!("{other}"));
                        self.must_exit = true;
                        self.pause = true;
                    }
                }
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
        let title = Line::from("Air Traffic Controller")
            .bold()
            .blue()
            .centered();
        let map: String = self.level.render();
        frame.render_widget(
            Paragraph::new(map).block(Block::bordered().title(title)),
            frame.area(),
        );
        if let Some(status_info) = self.status_info.take() {
            frame.render_widget(
                Paragraph::new(status_info).block(Block::bordered()),
                frame.area(),
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
        self.running = false;
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
