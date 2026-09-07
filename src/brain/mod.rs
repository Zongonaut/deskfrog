pub mod movement;

use movement::{Direction, MonitorBounds, Point, Wanderer};
use rand::Rng;

pub struct WorldInput<'a> {
    pub monitors: &'a [MonitorBounds],
}

pub struct Frog {
    pub pos: Point,
    wanderer: Wanderer,
}

impl Frog {
    pub fn new(start: Point, step_px: i32) -> Self {
        Self {
            pos: start,
            wanderer: Wanderer::new(step_px, Direction::E),
        }
    }

    pub fn step(&mut self, input: &WorldInput, rng: &mut impl Rng) {
        self.pos = self.wanderer.next_step(self.pos, input.monitors, rng);
    }
}
