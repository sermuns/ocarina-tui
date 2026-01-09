use ratatui::{
    style::Color,
    widgets::canvas::{self, Circle},
};
use rustysynth::MidiFile;
use std::io::Cursor;

pub const FULL_SOUNDFONT: &[u8] = include_bytes!("../assets/zelda3sf2/LttPSF2.sf2");
pub const OCARINA_ONLY_SOUNDFONT: &[u8] = include_bytes!("../assets/zelda3sf2/000_079 Ocarina.sf2");
pub const OPENING_SONG: &[u8] = include_bytes!("../assets/zelda3sf2/oot_opening.mid");

pub const NUM_NOTES: usize = 8;

#[derive(Debug)]
pub enum Song {
    BoleroOfFire,
    EponasSong,
    MinuetOfForest,
    NocturneOfShadow,
    PreludeOfLight,
    RequiemOfSpirit,
    SariasSong,
    SerenadeOfWater,
    SongOfStorms,
    SongOfTime,
    SunsSong,
    ZeldasLullaby,
}

impl Song {
    pub fn name(&self) -> &'static str {
        match self {
            Song::BoleroOfFire => "Bolero Of Fire",
            Song::EponasSong => "Eponas Song",
            Song::MinuetOfForest => "Minuet of Forest",
            Song::NocturneOfShadow => "Nocturne of Shadow",
            Song::PreludeOfLight => "Prelude of Light",
            Song::RequiemOfSpirit => "Requiem of Spirit",
            Song::SariasSong => "Sarias Song",
            Song::SerenadeOfWater => "Serenade of Water",
            Song::SongOfStorms => "Song of Storms",
            Song::SongOfTime => "Song of Time",
            Song::SunsSong => "Sun's Song",
            Song::ZeldasLullaby => "Zelda's Lullaby",
        }
    }
    pub fn midi_file(&self) -> MidiFile {
        let midi_bytes: &[u8] = match self {
            Song::BoleroOfFire => include_bytes!("../assets/zelda3sf2/oot_bolerooffire.mid"),
            Song::EponasSong => include_bytes!("../assets/zelda3sf2/oot_ocarina_eponassong.mid"),
            Song::MinuetOfForest => include_bytes!("../assets/zelda3sf2/oot_minofwood.mid"),
            Song::NocturneOfShadow => todo!(),
            Song::PreludeOfLight => todo!(),
            Song::RequiemOfSpirit => include_bytes!("../assets/zelda3sf2/oot_spiritual.mid"),
            Song::SariasSong => include_bytes!("../assets/zelda3sf2/oot_ocarina_saria.mid"),
            Song::SerenadeOfWater => todo!(),
            Song::SongOfStorms => {
                include_bytes!("../assets/zelda3sf2/oot_ocarina_songofstorms.mid")
            }
            Song::SongOfTime => include_bytes!("../assets/zelda3sf2/oot_ocarina_songoftime.mid"),
            Song::SunsSong => include_bytes!("../assets/zelda3sf2/oot_ocarina_sunssong.mid"),
            Song::ZeldasLullaby => {
                include_bytes!("../assets/zelda3sf2/oot_ocarina_zeldalullaby.mid")
            }
        };

        MidiFile::new(&mut Cursor::new(midi_bytes)).unwrap()
    }
}

pub fn song_from_notes(notes: &[Option<Note>; NUM_NOTES]) -> Option<Song> {
    use Note::*;
    match notes {
        [
            Some(Left),
            Some(Up),
            Some(Right),
            Some(Left),
            Some(Up),
            Some(Right),
            None,
            None,
        ] => Some(Song::ZeldasLullaby),
        [
            Some(Up),
            Some(Left),
            Some(Right),
            Some(Up),
            Some(Left),
            Some(Right),
            None,
            None,
        ] => Some(Song::EponasSong),
        [
            Some(Down),
            Some(Right),
            Some(Left),
            Some(Down),
            Some(Right),
            Some(Left),
            None,
            None,
        ] => Some(Song::SariasSong),
        [
            Some(Right),
            Some(Down),
            Some(Up),
            Some(Right),
            Some(Down),
            Some(Up),
            None,
            None,
        ] => Some(Song::SunsSong),
        [
            Some(Right),
            Some(A),
            Some(Down),
            Some(Right),
            Some(A),
            Some(Down),
            None,
            None,
        ] => Some(Song::SongOfTime),
        [
            Some(A),
            Some(Down),
            Some(Up),
            Some(A),
            Some(Down),
            Some(Up),
            None,
            None,
        ] => Some(Song::SongOfStorms),
        [
            Some(A),
            Some(Up),
            Some(Left),
            Some(Right),
            Some(Left),
            Some(Right),
            None,
            None,
        ] => Some(Song::MinuetOfForest),
        [
            Some(Down),
            Some(A),
            Some(Down),
            Some(A),
            Some(Right),
            Some(Down),
            Some(Right),
            Some(Down),
        ] => Some(Song::BoleroOfFire),
        [
            Some(A),
            Some(Down),
            Some(Right),
            Some(Right),
            Some(Left),
            None,
            None,
            None,
        ] => Some(Song::SerenadeOfWater),
        [
            Some(Left),
            Some(Right),
            Some(Right),
            Some(A),
            Some(Left),
            Some(Right),
            Some(Down),
            None,
        ] => Some(Song::NocturneOfShadow),
        [
            Some(A),
            Some(Down),
            Some(A),
            Some(Right),
            Some(Down),
            Some(A),
            None,
            None,
        ] => Some(Song::RequiemOfSpirit),
        [
            Some(Up),
            Some(Right),
            Some(Up),
            Some(Right),
            Some(Left),
            Some(Up),
            None,
            None,
        ] => Some(Song::PreludeOfLight),
        _ => None,
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Note {
    A,
    Down,
    Right,
    Left,
    Up,
}

impl Note {
    pub fn midi_key(&self) -> i32 {
        match self {
            Self::A => 62,
            Self::Down => 65,
            Self::Right => 69,
            Self::Left => 71,
            Self::Up => 74,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Note::A => "A",
            Note::Down => "▼",
            Note::Right => "▶",
            Note::Left => "◀",
            Note::Up => "▲",
        }
    }
}

impl Note {
    pub fn draw(self, ctx: &mut canvas::Context, x: f64, y: f64) {
        let color = if matches!(self, Note::A) {
            Color::Blue
        } else {
            Color::Yellow
        };
        ctx.draw(&Circle::new(x, y, 1.6, color));
        ctx.print(x + 0.2, y, self.symbol());
    }
}
