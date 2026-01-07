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

impl From<[NoteButton; NUM_NOTES]> for Song {
    fn from(value: [NoteButton; NUM_NOTES]) -> Self {
        use NoteButton as n;
        match value {
            [
                n::Right,
                n::A,
                n::Down,
                n::Right,
                n::A,
                n::Down,
                n::None,
                n::None,
            ] => Song::SongOfTime,
            [n::A, n::Down, n::Up, n::A, n::Down, n::Up, n::None, n::None] => Song::SongOfStorms,
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
