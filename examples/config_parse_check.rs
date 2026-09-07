//! Verifies the shipped example `deskfrog.toml` at the repo root actually
//! parses into the values it claims to, catching schema drift or the classic
//! TOML gotcha where a plain key placed after a [section] header silently
//! becomes part of that section instead of staying top-level.
fn main() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/deskfrog.toml"))
        .expect("read shipped example config");
    let config: deskfrog::config::Config = toml::from_str(&text).expect("parse shipped example config");

    println!("frog_size_px = {}", config.appearance.frog_size_px);
    println!("bubble_scale = {}", config.appearance.bubble_scale);
    println!("step_interval_ms = {}", config.behavior.step_interval_ms);
    println!("flee_radius_px = {}", config.behavior.flee_radius_px);
    println!("messages = {:?}", config.messages);

    assert_eq!(config.appearance.frog_size_px, 24.0);
    assert_eq!(config.appearance.bubble_scale, 1);
    assert_eq!(config.behavior.step_interval_ms, 500);
    assert_eq!(config.behavior.flee_radius_px, 120);
    assert_eq!(config.messages.len(), 7, "messages must parse as a top-level key, not get swallowed into [appearance]");
    println!("all assertions passed");
}
