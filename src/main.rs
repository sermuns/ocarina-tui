use clap::Parser;
use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::{
        self,
        event::{self, KeyCode},
    },
    prelude::*,
    widgets::{
        Block, BorderType, Borders, Padding,
        canvas::{Canvas, Line as CLine},
    },
};
use rustysynth::{MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::sync::{Arc, Mutex};
use std::{io::Cursor, time::Duration};
use tinyaudio::prelude::*;

use ocarina_tui::song::*;

pub struct App {
    quitting: bool,
    current_note: Option<Note>,
    notes_buffer: [Option<Note>; NUM_NOTES],
    note_idx: usize,
    playing_song: Option<Song>,
    song_sequencer: Arc<Mutex<MidiFileSequencer>>,
    ocarina_synth: Arc<Mutex<Synthesizer>>,

    // needs to be alive for duration of audio playback
    _output_device: OutputDevice,
}

#[derive(Parser)]
struct Args {}

fn main() -> Result<()> {
    color_eyre::install()?;
    let _args = Args::parse(); // TODO: add args, description etc
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
        let sound_font = Arc::new(SoundFont::new(&mut Cursor::new(FULL_SOUNDFONT)).unwrap());
        let settings = SynthesizerSettings::new(PARAMS.sample_rate as i32);

        let song_synth = Synthesizer::new(&sound_font, &settings).unwrap();
        let song_sequencer = Arc::new(Mutex::new(MidiFileSequencer::new(song_synth)));

        let ocarina_sound_font =
            Arc::new(SoundFont::new(&mut Cursor::new(OCARINA_ONLY_SOUNDFONT)).unwrap());
        let ocarina_synth = Arc::new(Mutex::new(
            Synthesizer::new(&ocarina_sound_font, &settings).unwrap(),
        ));

        let _output_device = run_output_device(PARAMS, {
            let song_sequencer = song_sequencer.clone();
            let ocarina_synth = ocarina_synth.clone();
            let mut left_ocarina = [0_f32; PARAMS.channel_sample_count];
            let mut right_ocarina = [0_f32; PARAMS.channel_sample_count];
            let mut left_song = [0_f32; PARAMS.channel_sample_count];
            let mut right_song = [0_f32; PARAMS.channel_sample_count];
            move |data| {
                if let Ok(mut ocarina_synth) = ocarina_synth.try_lock() {
                    ocarina_synth.render(&mut left_ocarina, &mut right_ocarina);
                }
                if let Ok(mut song_sequencer) = song_sequencer.try_lock() {
                    song_sequencer.render(&mut left_song, &mut right_song);
                };

                for (out, ((l_song, r_song), (l_ocarina, r_ocarina))) in
                    data.chunks_exact_mut(2).zip(
                        left_song
                            .iter()
                            .zip(right_song.iter())
                            .zip(left_ocarina.iter().zip(right_ocarina.iter())),
                    )
                {
                    const AMPLIFICATION: f32 = 3.0;
                    out[0] = (AMPLIFICATION * (*l_song + *l_ocarina)).tanh();
                    out[1] = (AMPLIFICATION * (*r_song + *r_ocarina)).tanh();
                }
            }
        })
        .unwrap();

        #[cfg(not(debug_assertions))]
        {
            let midi_file = Arc::new(MidiFile::new(&mut Cursor::new(OPENING_SONG)).unwrap());
            song_sequencer.lock().unwrap().play(&midi_file, false);
        }

        Ok(Self {
            quitting: false,
            playing_song: None,
            current_note: None,
            song_sequencer,
            ocarina_synth,
            _output_device,
            notes_buffer: [None; NUM_NOTES],
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
        frame.render_widget(
            title_block.clone().title(format!(
                " {:?} {:?} {:?}",
                self.current_note, self.note_idx, self.playing_song,
            )),
            footer,
        );
        #[cfg(not(debug_assertions))]
        frame.render_widget(title_block.clone(), footer);

        frame.render_widget(
            title_block.title(format!(" {} ", env!("CARGO_PKG_NAME"))),
            header,
        );

        let [_, message_area, canvas_outer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Min(1),
            Constraint::Length(20),
        ])
        .areas(body);

        if let Some(song) = &self.playing_song {
            let message_text =
                Line::from_iter(["You played ".into(), song.name().blue()]).centered();
            frame.render_widget(message_text, message_area);
        }

        let canvas_area = canvas_outer_area.centered_horizontally(Constraint::Max(100));
        let canvas = Canvas::default()
            .block(Block::bordered().padding(Padding::uniform(1)))
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
                    let Some(note) = note else {
                        continue;
                    };
                    let x = f64::from(note_spacing * (i as u16 + 1));
                    let y = 1.2 + note_height * f64::from(note as u8);
                    note.draw(ctx, x, y);
                }
            })
            .x_bounds([0., f64::from(canvas_area.width)])
            .y_bounds([0., f64::from(canvas_area.height)]);
        frame.render_widget(canvas, canvas_area);
    }

    fn do_note(&mut self, new_note: Option<Note>) {
        self.song_sequencer.lock().unwrap().stop();
        let mut ocarina_synth = self.ocarina_synth.lock().unwrap();

        const MIDI_CHANNEL: i32 = 0;
        const MIDI_VELOCITY: i32 = 100;
        if self.current_note != new_note {
            if let Some(current) = self.current_note {
                ocarina_synth.note_off(MIDI_CHANNEL, current.midi_key());
            }
            if let Some(new) = new_note {
                ocarina_synth.note_on(MIDI_CHANNEL, new.midi_key(), MIDI_VELOCITY);
            }
        }

        self.current_note = new_note;

        if new_note.is_none() {
            self.notes_buffer.fill(None);
            self.note_idx = 0;
            return;
        }

        if self.note_idx >= NUM_NOTES {
            self.notes_buffer.fill(None);
            self.note_idx = 0;
        }

        self.notes_buffer[self.note_idx] = new_note;
        self.note_idx += 1;

        let mut song_sequencer = self.song_sequencer.lock().unwrap();

        self.playing_song = song_from_notes(&self.notes_buffer);

        let Some(song) = &self.playing_song else {
            song_sequencer.stop();
            return;
        };

        song_sequencer.play(&Arc::new(song.midi_file()), false);
    }

    fn handle_events(&mut self) -> Result<()> {
        use crossterm::event::{Event, KeyModifiers};

        if !event::poll(Duration::from_millis(300))? {
            // TODO: ability to hold a note?
            if let Ok(mut ocarina_synth) = self.ocarina_synth.try_lock() {
                ocarina_synth.note_off_all(false);
            }
            return Ok(());
        }

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

        let new_note = match key_event.code {
            KeyCode::Char('a') => Some(Note::A),
            KeyCode::Down | KeyCode::Char('j') => Some(Note::Down),
            KeyCode::Right | KeyCode::Char('l') => Some(Note::Right),
            KeyCode::Left | KeyCode::Char('h') => Some(Note::Left),
            KeyCode::Up | KeyCode::Char('k') => Some(Note::Up),
            _ => None,
        };
        self.do_note(new_note);

        Ok(())
    }

    fn quit(&mut self) {
        self.quitting = true;
    }
}
