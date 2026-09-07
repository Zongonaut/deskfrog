use rand::{Rng, RngExt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl MonitorBounds {
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x < self.x + self.width && p.y >= self.y && p.y < self.y + self.height
    }
}

pub fn on_any_monitor(p: Point, monitors: &[MonitorBounds]) -> bool {
    monitors.iter().any(|m| m.contains(p))
}

pub fn monitor_containing(p: Point, monitors: &[MonitorBounds]) -> Option<&MonitorBounds> {
    monitors.iter().find(|m| m.contains(p))
}

fn distance_sq(a: Point, b: Point) -> i64 {
    let dx = (a.x - b.x) as i64;
    let dy = (a.y - b.y) as i64;
    dx * dx + dy * dy
}

pub fn within(a: Point, b: Point, radius_px: i32) -> bool {
    distance_sq(a, b) <= (radius_px as i64) * (radius_px as i64)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    const ALL: [Direction; 8] = [
        Direction::N,
        Direction::NE,
        Direction::E,
        Direction::SE,
        Direction::S,
        Direction::SW,
        Direction::W,
        Direction::NW,
    ];

    fn index(self) -> i32 {
        Self::ALL.iter().position(|d| *d == self).unwrap() as i32
    }

    fn delta(self) -> (i32, i32) {
        match self {
            Direction::N => (0, -1),
            Direction::NE => (1, -1),
            Direction::E => (1, 0),
            Direction::SE => (1, 1),
            Direction::S => (0, 1),
            Direction::SW => (-1, 1),
            Direction::W => (-1, 0),
            Direction::NW => (-1, -1),
        }
    }

    /// Turns by `steps` positions around the 8-direction compass (positive = clockwise).
    fn rotated(self, steps: i32) -> Direction {
        let n = Self::ALL.len() as i32;
        let idx = (self.index() + steps).rem_euclid(n) as usize;
        Self::ALL[idx]
    }

    fn random(rng: &mut impl Rng) -> Direction {
        Self::ALL[rng.random_range(0..Self::ALL.len())]
    }
}

/// Picks whichever adjacent grid cell ends up farthest from `threat` (the
/// cursor), staying on some monitor. Used for the frog's flee behavior — this
/// is a direct maximization rather than momentum-biased, so it always retreats
/// along the most effective heading available.
pub fn flee_step(pos: Point, threat: Point, step_px: i32, monitors: &[MonitorBounds]) -> Point {
    Direction::ALL
        .into_iter()
        .filter_map(|dir| {
            let (dx, dy) = dir.delta();
            let candidate = Point {
                x: pos.x + dx * step_px,
                y: pos.y + dy * step_px,
            };
            on_any_monitor(candidate, monitors).then(|| (candidate, distance_sq(candidate, threat)))
        })
        .max_by_key(|(_, score)| *score)
        .map(|(p, _)| p)
        .unwrap_or(pos)
}

/// Grid-based wandering with momentum: the frog is biased to keep heading the way
/// it was already going, occasionally drifts to an adjacent heading, and rarely
/// picks a fresh direction outright — this reads as purposeful roguelike movement
/// rather than a jittery random walk.
pub struct Wanderer {
    step_px: i32,
    dir: Direction,
}

impl Wanderer {
    pub fn new(step_px: i32, initial_dir: Direction) -> Self {
        Self {
            step_px,
            dir: initial_dir,
        }
    }

    pub fn step_px(&self) -> i32 {
        self.step_px
    }

    /// Picks the next grid cell. Falls back through progressively wider heading
    /// deviations when the preferred cell lands off every monitor (e.g. the dead
    /// space between two displays of different resolution), and stays put only if
    /// every direction is blocked.
    pub fn next_step(&mut self, pos: Point, monitors: &[MonitorBounds], rng: &mut impl Rng) -> Point {
        let roll: f32 = rng.random();
        let candidate_dir = if roll < 0.6 {
            self.dir
        } else if roll < 0.85 {
            self.dir.rotated(if rng.random_bool(0.5) { 1 } else { -1 })
        } else {
            Direction::random(rng)
        };

        for offset in [0, 1, -1, 2, -2, 3, -3, 4] {
            let dir = candidate_dir.rotated(offset);
            let (dx, dy) = dir.delta();
            let next = Point {
                x: pos.x + dx * self.step_px,
                y: pos.y + dy * self.step_px,
            };
            if on_any_monitor(next, monitors) {
                self.dir = dir;
                return next;
            }
        }
        pos
    }
}
