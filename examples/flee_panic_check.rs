//! Dev-only check for the panic-flee feature: repeatedly triggers a fresh flee
//! and reports how often it turns into a panic (bubble + double-speed).
use deskfrog::brain::movement::{flee_step_axis_aligned_multi, MonitorBounds, Point};
use deskfrog::brain::{Frog, WorldInput};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn check_axis_alignment() {
    let monitors = vec![MonitorBounds { x: 0, y: 0, width: 3840, height: 1600 }];
    let mut diagonal_moves = 0;
    let mut total = 0;
    // Sweep threats all the way around the frog, including exact-diagonal cases
    // where an 8-direction search would prefer a diagonal (tie-break risk).
    for angle_deg in (0..360).step_by(5) {
        let rad = (angle_deg as f64).to_radians();
        let threat = Point {
            x: 1000 + (200.0 * rad.cos()) as i32,
            y: 1000 + (200.0 * rad.sin()) as i32,
        };
        let pos = Point { x: 1000, y: 1000 };
        // 2 steps, same as a real panic tick — this is what must stay axis-pure.
        let next = flee_step_axis_aligned_multi(pos, threat, 24, 2, &monitors);
        let dx = next.x - pos.x;
        let dy = next.y - pos.y;
        total += 1;
        if dx != 0 && dy != 0 {
            diagonal_moves += 1;
            println!("angle {angle_deg}: DIAGONAL net move dx={dx} dy={dy}");
        }
    }
    println!("axis-aligned check (2-step net displacement): {diagonal_moves}/{total} diagonal (must be 0)");
}

fn main() {
    check_axis_alignment();

    let monitors = vec![MonitorBounds { x: 0, y: 0, width: 3840, height: 1600 }];
    let far_cursor = Point { x: 3000, y: 3000 };
    let start = Point { x: 1000, y: 1000 };

    let mut rng = StdRng::seed_from_u64(42);
    let mut frog = Frog::new(start, 24, &mut rng);

    let mut panics = 0;
    let mut trials = 0;
    let mut fastest_moves_seen = 0;

    for trial in 0..400 {
        // Approach: put the cursor right on the frog to force a fresh flee trigger.
        let close_cursor = frog.pos;
        let before = frog.pos;
        frog.step(&WorldInput { monitors: &monitors, cursor: close_cursor }, &mut rng);
        trials += 1;

        let moved = ((frog.pos.x - before.x).abs()).max((frog.pos.y - before.y).abs());
        if moved > 24 {
            fastest_moves_seen += 1;
        }

        if let Some(text) = frog.bubble() {
            if text == "Eeek! Go away!" {
                panics += 1;
                println!("trial {trial}: PANIC — bubble={text:?} moved={moved}px in one tick");
            }
        }

        // Let it settle back to Wander before the next approach.
        for _ in 0..20 {
            frog.step(&WorldInput { monitors: &monitors, cursor: far_cursor }, &mut rng);
        }
    }

    println!("panics: {panics}/{trials} fresh-flee triggers (double-speed ticks observed: {fastest_moves_seen})");

    // Regression check: panic must never interrupt a sleep or an in-progress
    // commentary bubble — only a plain, silent wander.
    let mut rng2 = StdRng::seed_from_u64(7);
    let mut frog2 = Frog::new(Point { x: 1000, y: 1000 }, 24, &mut rng2);
    let mut sleep_interrupts = 0;
    let mut commentary_interrupts = 0;
    let mut sleep_trials = 0;
    let mut commentary_trials = 0;

    for _ in 0..20000 {
        frog2.step(&WorldInput { monitors: &monitors, cursor: far_cursor }, &mut rng2);
        let bubble = frog2.bubble();
        let is_sleep = matches!(bubble.as_deref(), Some("z z z") | Some("zZzZz"));
        let is_commentary = matches!(&bubble, Some(t) if t != "z z z" && t != "zZzZz" && t != "!" && t != "Eeek! Go away!");

        if is_sleep || is_commentary {
            let close_cursor = frog2.pos;
            frog2.step(&WorldInput { monitors: &monitors, cursor: close_cursor }, &mut rng2);
            let after = frog2.bubble();
            let panicked = matches!(after.as_deref(), Some("Eeek! Go away!"));
            if is_sleep {
                sleep_trials += 1;
                if panicked {
                    sleep_interrupts += 1;
                }
            } else {
                commentary_trials += 1;
                if panicked {
                    commentary_interrupts += 1;
                }
            }
            // Walk away and let it settle before continuing to scan.
            for _ in 0..25 {
                frog2.step(&WorldInput { monitors: &monitors, cursor: far_cursor }, &mut rng2);
            }
        }
    }

    println!(
        "sleep interrupted by panic: {sleep_interrupts}/{sleep_trials}; commentary interrupted by panic: {commentary_interrupts}/{commentary_trials}"
    );

    // Idle/sleep timing check: classify each tick as sleep (bubble shows z z z),
    // idle (position didn't move and it's not a sleep tick), or moved.
    let mut rng3 = StdRng::seed_from_u64(99);
    let mut frog3 = Frog::new(Point { x: 1000, y: 1000 }, 24, &mut rng3);
    let (mut sleep_ticks, mut idle_ticks, mut moved_ticks) = (0u32, 0u32, 0u32);
    let (mut sleep_runs, mut idle_runs) = (0u32, 0u32);
    let mut prev_sleeping = false;
    let mut prev_idle = false;

    for _ in 0..20000 {
        let before = frog3.pos;
        frog3.step(&WorldInput { monitors: &monitors, cursor: far_cursor }, &mut rng3);
        let sleeping_now = matches!(frog3.bubble().as_deref(), Some("z z z") | Some("zZzZz"));
        let idle_now = !sleeping_now && frog3.pos == before;

        if sleeping_now {
            sleep_ticks += 1;
            if !prev_sleeping {
                sleep_runs += 1;
            }
        } else if idle_now {
            idle_ticks += 1;
            if !prev_idle {
                idle_runs += 1;
            }
        } else {
            moved_ticks += 1;
        }
        prev_sleeping = sleeping_now;
        prev_idle = idle_now;
    }

    let avg_idle_run = idle_ticks as f64 / idle_runs.max(1) as f64;
    let avg_sleep_run = sleep_ticks as f64 / sleep_runs.max(1) as f64;
    println!(
        "ticks: sleep={sleep_ticks} idle={idle_ticks} moved={moved_ticks}; idle_runs={idle_runs} (avg {avg_idle_run:.2} ticks), sleep_runs={sleep_runs} (avg {avg_sleep_run:.2} ticks), sleep/idle ratio={:.2}",
        avg_sleep_run / avg_idle_run
    );
}
