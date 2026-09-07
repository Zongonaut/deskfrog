use rand::{Rng, RngExt};

pub const MESSAGES: &[&str] = &[
    "Why is your mouse so fast?",
    "I have important frog business.",
    "Ribbit.",
    "Just vibing here.",
    "Is that a bug? Tasty.",
    "Don't mind me.",
    "Nice desktop.",
];

/// Ticks are whatever cadence the caller drives `tick()` at (one movement step,
/// in DeskFrog's case) — this stays decoupled from wall-clock time so the brain
/// has no OS dependency.
pub struct Commentary {
    messages: Vec<String>,
    ticks_until_next: u32,
    speaking_ticks_left: u32,
    current: Option<String>,
}

impl Commentary {
    pub fn new(messages: Vec<String>, rng: &mut impl Rng) -> Self {
        Self {
            messages,
            ticks_until_next: rng.random_range(4..10),
            speaking_ticks_left: 0,
            current: None,
        }
    }

    pub fn tick(&mut self, rng: &mut impl Rng) {
        if self.speaking_ticks_left > 0 {
            self.speaking_ticks_left -= 1;
            if self.speaking_ticks_left == 0 {
                self.current = None;
                self.ticks_until_next = rng.random_range(20..80);
            }
            return;
        }

        if self.ticks_until_next > 0 {
            self.ticks_until_next -= 1;
            return;
        }

        if self.messages.is_empty() {
            self.ticks_until_next = rng.random_range(20..80);
            return;
        }

        let msg = &self.messages[rng.random_range(0..self.messages.len())];
        self.current = Some(msg.clone());
        self.speaking_ticks_left = 6;
    }

    pub fn bubble(&self) -> Option<&str> {
        self.current.as_deref()
    }
}
