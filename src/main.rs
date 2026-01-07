use clap::Parser;
use color_eyre::Result;
use itertools::Itertools;
use ratatui::{
    DefaultTerminal,
    crossterm::{self, event::KeyCode},
    prelude::*,
    symbols::Marker,
    widgets::{
        Block, BorderType, Borders, Padding, Paragraph,
        canvas::{self, Canvas, Circle, Line as CLine, Shape},
    },
};
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::{
    fs::File,
    io::Cursor,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tinyaudio::prelude::*;

use ocarina_tui::song::*;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub struct App {
    quitting: bool,
    // stream_handle: OutputStream,
    // sink: Sink,
    current_note: NoteButton,
    notes_buffer: [NoteButton; NUM_NOTES],
    note_idx: usize,
    song_played: Song,

    message: String,
    /// when non-zero, counting down. clears `message` on completion.
    message_clear_timeout: Duration,
}

#[derive(Parser)]
struct Args {}

fn main() -> Result<()> {
    color_eyre::install()?;
    let _args = Args::parse();
    let mut app = App::new()?;
    ratatui::run(|terminal| app.run(terminal))
}

impl App {
    fn new() -> Result<Self> {
        // let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        // let sink = rodio::Sink::connect_new(stream_handle.mixer());
        // sink.append(NOTES.d_a.clone());

        Ok(Self {
            quitting: false,
            song_played: Song::None,
            // stream_handle,
            // sink,
            current_note: NoteButton::None,
            message: String::new(),
            message_clear_timeout: Duration::ZERO,
            notes_buffer: [NoteButton::None; NUM_NOTES],
            note_idx: 0,
        })
    }
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // let params = OutputDeviceParameters {
        //     channels_count: 2,
        //     sample_rate: 44100,
        //     channel_sample_count: 4410,
        // };
        // let sound_font = Arc::new(SoundFont::new(&mut Cursor::new(SF2)).unwrap());
        // let song_of_time_midi = Arc::new(MidiFile::new(&mut Cursor::new(SONG_OF_TIME)).unwrap());
        //
        // let mut left: Vec<f32> = vec![0_f32; params.channel_sample_count];
        // let mut right: Vec<f32> = vec![0_f32; params.channel_sample_count];
        //
        // let settings = SynthesizerSettings::new(params.sample_rate as i32);
        // let synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();
        // let mut sequencer = MidiFileSequencer::new(synthesizer);
        //
        // sequencer.play(&song_of_time_midi, false);
        //
        // let mut _device = run_output_device(params, {
        //     move |data| {
        //         sequencer.render(&mut left[..], &mut right[..]);
        //         for (i, value) in left.iter().interleave(right.iter()).enumerate() {
        //             data[i] = *value;
        //         }
        //     }
        // })
        // .unwrap();

        while !self.quitting {
            terminal.draw(|f| self.render(f))?;
            self.handle_events()?;
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
            title_block.clone().title(format!(
                " {} {:?} {:?}",
                <&str>::from(self.current_note),
                self.note_idx,
                self.song_played,
            )),
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
                let line_spacing = canvas_area.height / (NUM_LINES);
                let note_spacing = canvas_area.width / (NUM_NOTES as u16);
                let x1 = 0.;
                let x2 = f64::from(canvas_area.width);

                for i in 0..NUM_LINES {
                    let y = 3. + f64::from(line_spacing * i);
                    ctx.draw(&CLine::new(x1, y, x2, y, Color::LightRed));
                }

                let note_height = f64::from(canvas_area.height / (NUM_NOTES as u16)) * 2.0; // FIXME:random ass constant to make it bigger
                for (i, note) in self.notes_buffer.into_iter().enumerate() {
                    if matches!(note, NoteButton::None) {
                        continue;
                    }
                    let x = f64::from(note_spacing * (i as u16 + 1));
                    let y = 1.2 + note_height * f64::from(note as u8);
                    note.draw(ctx, x, y);
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
        } else if self.note_idx >= NUM_NOTES - 1 {
            self.notes_buffer.fill(NoteButton::None);
            self.note_idx = 0;
        }

        self.notes_buffer[self.note_idx] = note;
        self.note_idx += 1;

        self.song_played = self.notes_buffer.into();
    }

    fn handle_events(&mut self) -> Result<()> {
        use crossterm::event::{Event, KeyModifiers};

        let Event::Key(key_event) = crossterm::event::read()? else {
            return Ok(());
        };

        if !key_event.is_press() {
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
