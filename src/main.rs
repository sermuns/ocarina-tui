use clap::Parser;
use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::{self, event::KeyCode},
    prelude::*,
    symbols::Marker,
    widgets::{
        Block, BorderType, Borders, Padding, Paragraph,
        canvas::{Canvas, Circle, Line as CLine, Shape},
    },
};
use rodio::{
    OutputStream, Sink, Source,
    mixer::{self},
    source::{Amplify, SineWave},
};
use std::{sync::LazyLock, time::Duration};

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Parser)]
struct Args {}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut app = App::new()?;
    ratatui::run(|terminal| app.run(terminal))
}

const NUM_NOTES: usize = 8;

pub struct App {
    quitting: bool,
    stream_handle: OutputStream,
    sink: Sink,
    current_note: NoteButton,
    notes_buffer: [NoteButton; NUM_NOTES],
    note_idx: usize,

    message: String,
    /// when non-zero, counting down. clears `message` on completion.
    message_clear_timeout: Duration,
}

#[derive(Debug, Copy, Clone)]
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

impl From<NoteButton> for &str {
    fn from(value: NoteButton) -> Self {
        match value {
            NoteButton::A => "a",
            NoteButton::Down => "↓",
            NoteButton::Right => "→",
            NoteButton::Left => "←",
            NoteButton::Up => "↑",
            NoteButton::None => " ",
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

static NOTES: LazyLock<NoteSources> = LazyLock::new(|| NoteSources {
    d_a: SineWave::new(293.66).amplify_normalized(0.5),
    f_down: SineWave::new(349.23).amplify_normalized(0.5),
    a_right: SineWave::new(440.00).amplify_normalized(0.5),
    b_left: SineWave::new(493.88).amplify_normalized(0.5),
    d_up: SineWave::new(587.33).amplify_normalized(0.5),
});

impl App {
    fn new() -> Result<Self> {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::Sink::connect_new(stream_handle.mixer());

        Ok(Self {
            quitting: false,
            stream_handle,
            sink,
            current_note: NoteButton::None,
            message: String::new(),
            message_clear_timeout: Duration::ZERO,
            notes_buffer: [NoteButton::None; NUM_NOTES],
            note_idx: 0,
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
            Constraint::Fill(1),
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
                .title(format!(" {:?} {:?} ", self.current_note, self.note_idx)),
            footer,
        );
        frame.render_widget(title_block.title(format!(" {} ", PKG_NAME)), header);

        let [_, message_area, canvas_outer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Min(1),
            Constraint::Length(20),
        ])
        .areas(body);

        let message_text =
            Line::from(vec!["You played the ".into(), "Song of Time".blue()]).centered();
        frame.render_widget(message_text, message_area);

        let canvas_area = canvas_outer_area.centered_horizontally(Constraint::Max(100));
        let canvas = Canvas::default()
            .block(Block::bordered().padding(Padding::uniform(1)))
            // .marker(Marker::Dot)
            .paint(|ctx| {
                const NUM_LINES: u16 = 4;
                let line_spacing = canvas_area.height / (NUM_LINES - 1);
                let note_spacing = canvas_area.width / (NUM_NOTES as u16 - 1);
                let x1 = 0.;
                let x2 = f64::from(canvas_area.width);

                for i in 0..NUM_LINES {
                    let y = f64::from(line_spacing * i);
                    ctx.draw(&CLine::new(x1, y, x2, y, Color::LightRed));
                }

                let note_height = f64::from(canvas_area.height / (NUM_NOTES as u16 - 3));
                for (i, note) in self.notes_buffer.into_iter().enumerate() {
                    if matches!(note, NoteButton::None) {
                        continue;
                    }
                    let x = f64::from(note_spacing * i as u16);
                    let y = note_height * f64::from(note as u8);
                    const NOTE_CIRCLE_RADIUS: f64 = 2.2;
                    ctx.draw(&Circle::new(x, y, NOTE_CIRCLE_RADIUS, Color::Yellow));
                    ctx.print::<&str>(x, y, (note).into());
                }
            })
            .x_bounds([0., f64::from(canvas_area.width)])
            .y_bounds([0., f64::from(canvas_area.height)]);
        frame.render_widget(canvas, canvas_area);
    }

    fn do_note(&mut self, note: NoteButton) {
        self.current_note = note;

        if matches!(note, NoteButton::None) {
            self.notes_buffer.fill(NoteButton::None);
            self.note_idx = 0;
            return;
        }

        self.notes_buffer[self.note_idx] = note;

        if self.note_idx >= NUM_NOTES - 1 {
            self.note_idx = 0;
        } else {
            self.note_idx += 1;
        }
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

        self.do_note(key_event.code.into());

        Ok(())
    }

    fn quit(&mut self) {
        self.quitting = true;
    }
}
