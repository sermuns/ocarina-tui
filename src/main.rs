use clap::Parser;
use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::{self, event::KeyCode},
    prelude::*,
    symbols::Marker,
    widgets::{
        Block, BorderType, Borders,
        canvas::{self, Canvas},
    },
};
use rodio::{
    OutputStream, Sink, Source,
    mixer::{self},
    source::{Amplify, SineWave},
};
use std::sync::LazyLock;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Parser)]
struct Args {}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut app = App::new()?;
    ratatui::run(|terminal| app.run(terminal))
}

pub struct App {
    quitting: bool,
    stream_handle: OutputStream,
    sink: Sink,
    note_pressed: NoteButton,
}

#[derive(Debug)]
enum NoteButton {
    A,
    Down,
    Right,
    Left,
    Up,
    None,
}

impl From<KeyCode> for NoteButton {
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::Char('a') => NoteButton::A,
            KeyCode::Down | KeyCode::Char('j') => NoteButton::Down,
            KeyCode::Right | KeyCode::Char('l') => NoteButton::Right,
            KeyCode::Left | KeyCode::Char('h') => NoteButton::Left,
            KeyCode::Up | KeyCode::Char('k') => NoteButton::Up,
            _ => NoteButton::None,
        }
    }
}

struct NoteSources {
    d_a: Amplify<SineWave>,
    f_down: Amplify<SineWave>,
    a_right: Amplify<SineWave>,
    b_left: Amplify<SineWave>,
    d_up: Amplify<SineWave>,
}

impl NoteSources {
    fn new() -> Self {
        NoteSources {
            d_a: SineWave::new(293.66).amplify_normalized(0.5),
            f_down: SineWave::new(349.23).amplify_normalized(0.5),
            a_right: SineWave::new(440.00).amplify_normalized(0.5),
            b_left: SineWave::new(493.88).amplify_normalized(0.5),
            d_up: SineWave::new(587.33).amplify_normalized(0.5),
        }
    }
}

static NOTES: LazyLock<NoteSources> = LazyLock::new(NoteSources::new);

impl App {
    fn new() -> Result<Self> {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::Sink::connect_new(stream_handle.mixer());

        Ok(Self {
            quitting: false,
            stream_handle,
            sink,
            note_pressed: NoteButton::None,
        })
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quitting {
            terminal.draw(|f| self.render(f))?;
            self.handle_events()?;
            // self.play_notes()?;
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        let title_block = Block::new()
            .title_style(Style::default().bold())
            .title_alignment(HorizontalAlignment::Center)
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::LightBlue))
            .borders(Borders::TOP);
        frame.render_widget(
            title_block
                .clone()
                .title(format!(" {:?} ", self.note_pressed)),
            footer,
        );
        frame.render_widget(title_block.title(format!(" {} ", PKG_NAME)), header);

        let canvas = Canvas::default()
            .block(Block::bordered())
            .marker(Marker::Dot)
            .paint(|ctx| {
                const LINE_SPACING: i16 = 3;
                let x2 = f64::from(body.width);
                let center = body.height / 2;
                for i in -4..=-1 {
                    let y = (center as i16 + LINE_SPACING * i).into();
                    ctx.draw(&canvas::Line::new(0., y, x2, y, Color::Gray));
                }
            })
            .x_bounds([0., f64::from(body.width)])
            .y_bounds([0., f64::from(body.height)]);
        frame.render_widget(canvas, body);
    }

    fn handle_events(&mut self) -> Result<()> {
        use crossterm::event::{Event, KeyEventKind, KeyModifiers};

        let Event::Key(key_event) = crossterm::event::read()? else {
            return Ok(());
        };

        if key_event.kind != KeyEventKind::Press {
            return Ok(());
        }

        if key_event.code == KeyCode::Char('q')
            || (key_event.modifiers == KeyModifiers::CONTROL
                && key_event.code == KeyCode::Char('c'))
        {
            self.quit();
        }

        let (_controller, _mixer) = mixer::mixer(2, 44_100);

        self.note_pressed = key_event.code.into();
        Ok(())
    }

    fn quit(&mut self) {
        self.quitting = true;
    }
}
