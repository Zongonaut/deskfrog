pub mod messages;
pub mod movement;

use messages::Commentary;
use movement::{within, Direction, MonitorBounds, Point, Wanderer};
use rand::{Rng, RngExt};

/// How much farther than the flee radius a sleeping frog's wake radius and the
/// fleeing frog's stop-fleeing distance are — preserves the tuned feel of these
/// margins regardless of the configured flee radius.
const WAKE_RADIUS_MARGIN_PX: i32 = 80;
const SAFE_DISTANCE_MARGIN_PX: i32 = 100;
/// Chance that a fresh flee turns into a panic (double speed + a startled shout).
const PANIC_CHANCE: f32 = 0.2;

const IDLE_TICKS_MIN: u32 = 2;
const IDLE_TICKS_MAX_EXCLUSIVE: u32 = 5;
/// Sleep naps run roughly 3-4x longer than an ordinary idle pause.
const SLEEP_TICKS_MIN: u32 = IDLE_TICKS_MIN * 5;
const SLEEP_TICKS_MAX_EXCLUSIVE: u32 = IDLE_TICKS_MAX_EXCLUSIVE * 6;

pub struct WorldInput<'a> {
    pub monitors: &'a [MonitorBounds],
    pub cursor: Point,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Wander,
    Idle(u32),
    Sleeping(u32),
    Startled(u32),
    Fleeing,
}

pub struct Frog {
    pub pos: Point,
    wanderer: Wanderer,
    commentary: Commentary,
    state: State,
    sleep_frame: u32,
    panic_ticks_left: u32,
    flee_radius_px: i32,
    wake_radius_px: i32,
    safe_distance_px: i32,
}

impl Frog {
    pub fn new(start: Point, step_px: i32, flee_radius_px: i32, messages: Vec<String>, rng: &mut impl Rng) -> Self {
        Self {
            pos: start,
            wanderer: Wanderer::new(step_px, Direction::E),
            commentary: Commentary::new(messages, rng),
            state: State::Wander,
            sleep_frame: 0,
            panic_ticks_left: 0,
            flee_radius_px,
            wake_radius_px: flee_radius_px + WAKE_RADIUS_MARGIN_PX,
            safe_distance_px: flee_radius_px + SAFE_DISTANCE_MARGIN_PX,
        }
    }

    pub fn step(&mut self, input: &WorldInput, rng: &mut impl Rng) {
        let threat_close = within(self.pos, input.cursor, self.flee_radius_px);
        let sleeping = matches!(self.state, State::Sleeping(_));
        // Panic only interrupts plain wandering — never a sleep, startle, idle
        // pause, or an in-progress commentary bubble, where an "Eeek!" popping
        // in on top would just look like a rendering glitch.
        let plain_wander =
            matches!(self.state, State::Wander) && self.commentary.bubble().is_none();
        if threat_close || (sleeping && within(self.pos, input.cursor, self.wake_radius_px)) {
            if plain_wander && rng.random::<f32>() < PANIC_CHANCE {
                self.panic_ticks_left = rng.random_range(2..4);
            }
            self.state = State::Fleeing;
        }

        match self.state {
            State::Fleeing => {
                self.pos = if self.panic_ticks_left > 0 {
                    movement::flee_step_axis_aligned_multi(self.pos, input.cursor, self.wanderer.step_px(), 2, input.monitors)
                } else {
                    movement::flee_step(self.pos, input.cursor, self.wanderer.step_px(), input.monitors)
                };
                self.panic_ticks_left = self.panic_ticks_left.saturating_sub(1);
                if !within(self.pos, input.cursor, self.safe_distance_px) {
                    self.state = State::Wander;
                    self.panic_ticks_left = 0;
                }
            }
            State::Idle(ticks_left) => {
                self.state = next_countdown(ticks_left, State::Wander, State::Idle);
            }
            State::Sleeping(ticks_left) => {
                self.sleep_frame = self.sleep_frame.wrapping_add(1);
                self.state = next_countdown(ticks_left, State::Wander, State::Sleeping);
            }
            State::Startled(ticks_left) => {
                self.state = next_countdown(ticks_left, State::Wander, State::Startled);
            }
            State::Wander => {
                let roll: f32 = rng.random();
                if roll < 0.05 {
                    self.state = State::Sleeping(rng.random_range(SLEEP_TICKS_MIN..SLEEP_TICKS_MAX_EXCLUSIVE));
                } else if roll < 0.08 {
                    self.state = State::Startled(1);
                } else if roll < 0.33 {
                    self.state = State::Idle(rng.random_range(IDLE_TICKS_MIN..IDLE_TICKS_MAX_EXCLUSIVE));
                } else {
                    self.pos = self.wanderer.next_step(self.pos, input.monitors, rng);
                }
            }
        }

        self.commentary.tick(rng);
    }

    /// The bubble text to show this tick, if any. Sleep and startle emotes take
    /// priority over ordinary commentary — the frog doesn't chit-chat mid-nap.
    pub fn bubble(&self) -> Option<String> {
        match self.state {
            State::Fleeing if self.panic_ticks_left > 0 => Some("Eeek! Go away!".to_string()),
            State::Sleeping(_) => Some(if self.sleep_frame % 2 == 0 { "z z z" } else { "zZzZz" }.to_string()),
            State::Startled(_) => Some("!".to_string()),
            _ => self.commentary.bubble().map(str::to_string),
        }
    }
}

fn next_countdown(ticks_left: u32, done: State, still_going: impl Fn(u32) -> State) -> State {
    if ticks_left <= 1 {
        done
    } else {
        still_going(ticks_left - 1)
    }
}
