pub mod messages;
pub mod movement;

use messages::Commentary;
use movement::{within, Direction, MonitorBounds, Point, Wanderer};
use rand::{Rng, RngExt};

const FLEE_RADIUS_PX: i32 = 120;
/// A sleeping frog wakes (and immediately flees) if the cursor gets this close —
/// wider than FLEE_RADIUS_PX so waking up doesn't feel like a jump-scare dodge.
const WAKE_RADIUS_PX: i32 = 200;
/// Stop fleeing once this far from the cursor.
const SAFE_DISTANCE_PX: i32 = 220;

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
}

impl Frog {
    pub fn new(start: Point, step_px: i32, rng: &mut impl Rng) -> Self {
        Self {
            pos: start,
            wanderer: Wanderer::new(step_px, Direction::E),
            commentary: Commentary::new(rng),
            state: State::Wander,
            sleep_frame: 0,
        }
    }

    pub fn step(&mut self, input: &WorldInput, rng: &mut impl Rng) {
        let threat_close = within(self.pos, input.cursor, FLEE_RADIUS_PX);
        let sleeping = matches!(self.state, State::Sleeping(_));
        if threat_close || (sleeping && within(self.pos, input.cursor, WAKE_RADIUS_PX)) {
            self.state = State::Fleeing;
        }

        match self.state {
            State::Fleeing => {
                self.pos = movement::flee_step(self.pos, input.cursor, self.wanderer.step_px(), input.monitors);
                if !within(self.pos, input.cursor, SAFE_DISTANCE_PX) {
                    self.state = State::Wander;
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
                    self.state = State::Sleeping(rng.random_range(6..14));
                } else if roll < 0.08 {
                    self.state = State::Startled(1);
                } else if roll < 0.20 {
                    self.state = State::Idle(rng.random_range(1..3));
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
