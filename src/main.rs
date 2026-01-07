use clap::Parser;
use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::{self, event::KeyCode},
    prelude::*,
    widgets::{
        Block, BorderType, Borders, Padding,
        canvas::{Canvas, Line as CLine},
    },
};
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::sync::{Arc, Mutex};
use std::{io::Cursor, time::Duration};
use tinyaudio::prelude::*;

use ocarina_tui::song::*;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub struct App {
    quitting: bool,
    // stream_handle: OutputStream,
    // sink: Sink,
    current_note: Arc<Mutex<NoteButton>>,
    notes_buffer: [NoteButton; NUM_NOTES],
    note_idx: usize,

    song_played: Song,
    song_sequencer: Arc<Mutex<MidiFileSequencer>>,

    output_device: OutputDevice,

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
        const PARAMS: OutputDeviceParameters = OutputDeviceParameters {
            channels_count: 2,
            sample_rate: 44100,
            channel_sample_count: 4410,
        };
        let sound_font = Arc::new(SoundFont::new(&mut Cursor::new(SF2)).unwrap());
        let settings = SynthesizerSettings::new(PARAMS.sample_rate as i32);

        let song_synth = Synthesizer::new(&sound_font, &settings).unwrap();
        let song_sequencer = Arc::new(Mutex::new(MidiFileSequencer::new(song_synth)));

        let mut left = [0_f32; PARAMS.channel_sample_count];
        let mut right = [0_f32; PARAMS.channel_sample_count];

        let current_note = Arc::new(Mutex::new(NoteButton::None));

        let output_device = run_output_device(PARAMS, {
            let song_sequencer = song_sequencer.clone();
            let current_note = current_note.clone();
            let mut ocarina_synth = Synthesizer::new(&sound_font, &settings).unwrap();
            move |data| {
                if let Ok(note) = current_note.try_lock()
                    && note.is_some()
                {
                    ocarina_synth.note_on(0, 60, 100);
                    ocarina_synth.render(&mut left, &mut right);
                } else if let Ok(mut song_sequencer) = song_sequencer.try_lock() {
                    song_sequencer.render(&mut left, &mut right);
                };

                for (out, (l, r)) in data.chunks_exact_mut(2).zip(left.iter().zip(right.iter())) {
                    out[0] = *l;
                    out[1] = *r;
                }
            }
        })
        .unwrap();

        // sleep(Duration::from_secs(10));

        let midi_file = Arc::new(MidiFile::new(&mut Cursor::new(OPENING_SONG)).unwrap());
        song_sequencer.lock().unwrap().play(&midi_file, false);

        Ok(Self {
            quitting: false,
            song_played: Song::None,
            current_note,
            song_sequencer,
            output_device,
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

        #[cfg(debug_assertions)]
        if let Ok(current_note) = self.current_note.try_lock() {
            frame.render_widget(
                title_block.clone().title(format!(
                    " {} {:?} {:?}",
                    <&str>::from(*current_note),
                    self.note_idx,
                    self.song_played,
                )),
                footer,
            );
        }
        frame.render_widget(title_block.title(format!(" {} ", PKG_NAME)), header);

        let [_, message_area, canvas_outer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Min(1),
            Constraint::Length(20),
        ])
        .areas(body);

        if self.song_played.is_some() {
            let message_text =
                Line::from_iter(["You played ".into(), <&str>::from(&self.song_played).blue()])
                    .centered();
            frame.render_widget(message_text, message_area);
        }

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
        self.song_sequencer.lock().unwrap().stop();

        *self.current_note.lock().unwrap() = note;

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

        let mut sequencer = self.song_sequencer.lock().unwrap();
        if self.song_played.is_some() {
            sequencer.play(&<Arc<MidiFile>>::from(&self.song_played), false);
        } else {
            sequencer.stop();
        }
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
