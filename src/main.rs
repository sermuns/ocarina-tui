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
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::sync::{Arc, Mutex};
use std::{io::Cursor, time::Duration};
use tinyaudio::prelude::*;

use ocarina_tui::song::*;

pub struct App {
    quitting: bool,
    current_note: NoteButton,
    notes_buffer: [NoteButton; NUM_NOTES],
    note_idx: usize,
    playing_song: Song,
    song_sequencer: Arc<Mutex<MidiFileSequencer>>,
    ocarina_synth: Arc<Mutex<Synthesizer>>,

    // needs to be alive for duration of audio playback
    _output_device: OutputDevice,
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
            playing_song: Song::None,
            current_note: NoteButton::None,
            song_sequencer,
            ocarina_synth,
            _output_device,
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
        frame.render_widget(
            title_block.clone().title(format!(
                " {} {:?} {:?}",
                <&str>::from(self.current_note),
                self.note_idx,
                self.playing_song,
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

        if self.playing_song.is_some() {
            let message_text = Line::from_iter([
                "You played ".into(),
                <&str>::from(&self.playing_song).blue(),
            ])
            .centered();
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

    fn do_note(&mut self, new_note: NoteButton) {
        self.song_sequencer.lock().unwrap().stop();

        let mut ocarina_synth = self.ocarina_synth.lock().unwrap();
        const MIDI_CHANNEL: i32 = 0;
        const MIDI_VELOCITY: i32 = 100;
        if self.current_note.is_none() && new_note.is_some() {
            ocarina_synth.note_on(MIDI_CHANNEL, new_note.midi_key(), MIDI_VELOCITY);
        } else if self.current_note.is_some() && new_note.is_none() {
            ocarina_synth.note_off(MIDI_CHANNEL, self.current_note.midi_key());
        } else if self.current_note.is_some() && new_note.is_some() && self.current_note != new_note
        {
            ocarina_synth.note_off(MIDI_CHANNEL, self.current_note.midi_key());
            ocarina_synth.note_on(MIDI_CHANNEL, new_note.midi_key(), MIDI_VELOCITY);
        }

        self.current_note = new_note;

        if !new_note.is_some() {
            self.notes_buffer.fill(NoteButton::None);
            self.note_idx = 0;
            return;
        }

        if self.note_idx >= NUM_NOTES {
            self.notes_buffer.fill(NoteButton::None);
            self.note_idx = 0;
        }

        self.notes_buffer[self.note_idx] = new_note;
        self.note_idx += 1;

        let song: Song = self.notes_buffer.into();

        let mut song_sequencer = self.song_sequencer.lock().unwrap();
        if song.is_some() {
            song_sequencer.play(&<Arc<MidiFile>>::from(&song), false);
        } else {
            song_sequencer.stop();
        }
        self.playing_song = song;
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

        self.do_note(key_event.code.into());

        Ok(())
    }

    fn quit(&mut self) {
        self.quitting = true;
    }
}
