use ratatui::{
    crossterm::event::KeyCode,
    style::Color,
    widgets::canvas::{self, Circle},
};
use rustysynth::MidiFile;
use std::{io::Cursor, sync::Arc};

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
    None,
}

impl From<&Song> for Arc<MidiFile> {
    fn from(value: &Song) -> Self {
        let bytes: &[u8] = match value {
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
            Song::None => panic!("tried to get song data for `Song::None`"),
        };
        Arc::new(MidiFile::new(&mut Cursor::new(bytes)).unwrap())
    }
}

impl Song {
    pub const fn is_none(&self) -> bool {
        matches!(self, Song::None)
    }
    pub const fn is_some(&self) -> bool {
        !self.is_none()
    }
}

impl From<&Song> for &str {
    fn from(value: &Song) -> Self {
        match value {
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
            Song::None => panic!("tried to stringify `Song::None`"),
        }
    }
}

impl From<[NoteButton; NUM_NOTES]> for Song {
    fn from(value: [NoteButton; NUM_NOTES]) -> Self {
        use NoteButton::*;
        match value {
            [Left, Up, Right, Left, Up, Right, None, None] => Song::ZeldasLullaby,
            [Up, Left, Right, Up, Left, Right, None, None] => Song::EponasSong,
            [Down, Right, Left, Down, Right, Left, None, None] => Song::SariasSong,
            [Right, Down, Up, Right, Down, Up, None, None] => Song::SunsSong,
            [Right, A, Down, Right, A, Down, None, None] => Song::SongOfTime,
            [A, Down, Up, A, Down, Up, None, None] => Song::SongOfStorms,
            [A, Up, Left, Right, Left, Right, None, None] => Song::MinuetOfForest,
            [Down, A, Down, A, Right, Down, Right, Down] => Song::BoleroOfFire,
            [A, Down, Right, Right, Left, None, None, None] => Song::SerenadeOfWater,
            [Left, Right, Right, A, Left, Right, Down, None] => Song::NocturneOfShadow,
            [A, Down, A, Right, Down, A, None, None] => Song::RequiemOfSpirit,
            [Up, Right, Up, Right, Left, Up, None, None] => Song::PreludeOfLight,
            _ => Song::None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NoteButton {
    A,
    Down,
    Right,
    Left,
    Up,
    None,
}

impl NoteButton {
    pub const fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn midi_key(&self) -> i32 {
        match self {
            Self::A => 62,
            Self::Down => 65,
            Self::Right => 69,
            Self::Left => 71,
            Self::Up => 74,
            Self::None => panic!("tried to get midi key for NoteButton::None"),
        }
    }
}

impl From<NoteButton> for &str {
    fn from(value: NoteButton) -> Self {
        match value {
            NoteButton::A => "A",
            NoteButton::Down => "▼",
            NoteButton::Right => "▶",
            NoteButton::Left => "◀",
            NoteButton::Up => "▲",
            NoteButton::None => " ",
        }
    }
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

impl NoteButton {
    pub fn draw(self, ctx: &mut canvas::Context, x: f64, y: f64) {
        let color = if matches!(self, NoteButton::A) {
            Color::Blue
        } else {
            Color::Yellow
        };
        ctx.draw(&Circle::new(x, y, 1.6, color));
        ctx.print::<&str>(x + 0.2, y, self.into());
    }
}
