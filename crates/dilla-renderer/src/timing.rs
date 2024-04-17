#![allow(unused)]

cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        use std::time::Instant;
    }
}

use std::fmt;

#[derive(Debug, Clone)]
pub struct Timing {
    time: f32,
    level: i8,
    message: String,
}

impl Timing {
    pub fn new(time: f32, level: i8, message: String) -> Self {
        Self {
            time,
            level,
            message,
        }
    }

    pub fn time(&self) -> f32 {
        self.time
    }
    pub fn level(&self) -> i8 {
        self.level
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[allow(unreachable_code)]
impl fmt::Display for Timing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[cfg(target_family = "wasm")]
        return write!(f, "");

        let time = format!("{:.3}", self.time());
        let mut level = String::new();
        for _ in 0..self.level() {
            level.push('\t');
        }
        write!(f, "{} | {}{}", time, level, self.message())
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn s() -> Instant {
    Instant::now()
}

#[cfg(not(target_family = "wasm"))]
pub fn t(now: Instant) -> f32 {
    now.elapsed().as_micros() as f32 / 1000.0
}

#[cfg(not(target_family = "wasm"))]
pub fn p(now: Instant, label: &str) {
    println!(
        "{}: {} ms",
        label,
        now.elapsed().as_micros() as f32 / 1000.0
    );
}

#[cfg(not(target_family = "wasm"))]
pub fn pt(now: Instant, label: &str, time: f32) {
    let elapsed = now.elapsed().as_micros() as f32 / 1000.0;
    if elapsed > time {
        println!("{}: {} ms", label, elapsed);
    }
}

#[cfg(target_family = "wasm")]
pub fn s() -> i8 {
    0
}

#[cfg(target_family = "wasm")]
pub fn t(_now: i8) -> f32 {
    0.0
}
