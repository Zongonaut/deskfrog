fn main() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug/deskfrog.toml"))
        .expect("read test config");
    let config: deskfrog::config::Config = toml::from_str(&text).expect("parse test config");

    println!("frog_size_px = {}", config.appearance.frog_size_px);
    println!("bubble_scale = {}", config.appearance.bubble_scale);
    println!("step_interval_ms = {}", config.behavior.step_interval_ms);
    println!("flee_radius_px = {}", config.behavior.flee_radius_px);
    println!("messages = {:?}", config.messages);

    assert_eq!(config.appearance.frog_size_px, 48.0);
    assert_eq!(config.appearance.bubble_scale, 2);
    assert_eq!(config.behavior.flee_radius_px, 120);
    assert_eq!(config.messages.len(), 1);
    assert_eq!(config.messages[0], "CONFIG TEST MESSAGE WORKS");
    println!("all assertions passed");
}
