use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub appearance: Appearance,
    pub behavior: Behavior,
    pub messages: Vec<String>,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Appearance {
    pub frog_size_px: f32,
    pub bubble_scale: u32,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Behavior {
    pub step_interval_ms: u32,
    pub flee_radius_px: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            appearance: Appearance::default(),
            behavior: Behavior::default(),
            messages: crate::brain::messages::MESSAGES.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            frog_size_px: 24.0,
            bubble_scale: 1,
        }
    }
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            step_interval_ms: 500,
            flee_radius_px: 120,
        }
    }
}

/// Loads `deskfrog.toml` from next to the running executable, falling back to
/// built-in defaults if it's absent or fails to parse — a bad or missing config
/// must never prevent startup.
pub fn load() -> Config {
    let path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("deskfrog.toml")));

    let Some(path) = path else {
        return Config::default();
    };

    match std::fs::read_to_string(&path) {
        Ok(text) => match toml::from_str(&text) {
            Ok(config) => config,
            Err(err) => {
                eprintln!("deskfrog.toml: {err} — using defaults");
                Config::default()
            }
        },
        Err(_) => Config::default(),
    }
}
