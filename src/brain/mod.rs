pub mod messages;
pub mod movement;

use messages::Commentary;
use movement::{Direction, MonitorBounds, Point, Wanderer};
use rand::Rng;

pub struct WorldInput<'a> {
    pub monitors: &'a [MonitorBounds],
}

pub struct Frog {
    pub pos: Point,
    wanderer: Wanderer,
    commentary: Commentary,
}

impl Frog {
    pub fn new(start: Point, step_px: i32, rng: &mut impl Rng) -> Self {
        Self {
            pos: start,
            wanderer: Wanderer::new(step_px, Direction::E),
            commentary: Commentary::new(rng),
        }
    }

    pub fn step(&mut self, input: &WorldInput, rng: &mut impl Rng) {
        self.pos = self.wanderer.next_step(self.pos, input.monitors, rng);
        self.commentary.tick(rng);
    }

    pub fn bubble(&self) -> Option<&str> {
        self.commentary.bubble()
    }
}
