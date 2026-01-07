use ratatui::{
    crossterm::event::KeyCode,
    style::Color,
    widgets::canvas::{self, Circle},
};

pub const SF2: &[u8] = include_bytes!("../assets/zelda3sf2/LttPSF2.sf2");
pub const SONG_OF_TIME: &[u8] = include_bytes!("../assets/zelda3sf2/oot_ocarina_songoftime.mid");

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

impl Song {
    pub fn is_some(&self) -> bool {
        !matches!(self, Song::None)
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
            [Down, A, Down, A, Right, Down, Left, Down] => Song::BoleroOfFire,
            [A, Down, Right, Right, Left, None, None, None] => Song::SerenadeOfWater,
            [Left, Right, Right, A, Left, Right, Down, None] => Song::NocturneOfShadow,
            [A, Down, A, Right, Down, A, None, None] => Song::RequiemOfSpirit,
            [Up, Right, Up, Right, Left, Up, None, None] => Song::PreludeOfLight,
            _ => Song::None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NoteButton {
    A,
    Down,
    Right,
    Left,
    Up,
    None,
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
        const NOTE_CIRCLE_RADIUS: f64 = 1.4;

        let color = match self {
            NoteButton::A => Color::Blue,
            _ => Color::Yellow,
        };

        for c in [1.0 /* 0.9, 0.8, 0.7, 0.6, 0.5, 0.2, 0.1*/] {
            ctx.draw(&Circle::new(x, y, NOTE_CIRCLE_RADIUS * c, color));
        }
        ctx.print::<&str>(x + 0.2, y, self.into());
    }
}
