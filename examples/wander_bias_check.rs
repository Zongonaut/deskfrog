//! Dev-only check for the cardinal/diagonal wander bias.
use deskfrog::brain::movement::{Direction, MonitorBounds, Point, Wanderer};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn main() {
    let monitors = vec![MonitorBounds { x: 0, y: 0, width: 100_000, height: 100_000 }];
    let mut rng = StdRng::seed_from_u64(123);
    let mut wanderer = Wanderer::new(24, Direction::E);
    let mut pos = Point { x: 50_000, y: 50_000 };

    let mut cardinal = 0u32;
    let mut diagonal = 0u32;

    for _ in 0..100_000 {
        let next = wanderer.next_step(pos, &monitors, &mut rng);
        let dx = (next.x - pos.x).abs();
        let dy = (next.y - pos.y).abs();
        if dx != 0 && dy != 0 {
            diagonal += 1;
        } else {
            cardinal += 1;
        }
        pos = next;
    }

    let total = cardinal + diagonal;
    println!(
        "cardinal={cardinal} ({:.1}%)  diagonal={diagonal} ({:.1}%)",
        100.0 * cardinal as f64 / total as f64,
        100.0 * diagonal as f64 / total as f64
    );
}
