//! Live service readiness probes for player-facing banners.
//!
//! These are soft UI signals (never stubs). They mirror the launcher's honesty
//! bar without spawning Foundry.

use serde_json::Value;
use std::time::Duration;

use super::chronos::{comfyui_url, DIRECTOR_URL};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Up,
    Down,
    Unknown,
}

impl ServiceState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessSnapshot {
    pub ollama: ServiceState,
    pub director: ServiceState,
    pub comfy: ServiceState,
}

impl ReadinessSnapshot {
    pub fn banner_line(&self) -> String {
        format!(
            "Ollama {} · Chronos Director {} · Comfy {}",
            self.ollama.label(),
            self.director.label(),
            self.comfy.label()
        )
    }

    pub fn chronos_ready(&self) -> bool {
        self.director == ServiceState::Up && self.comfy == ServiceState::Up
    }

    pub fn player_hint(&self) -> Option<&'static str> {
        if self.chronos_ready() && self.ollama == ServiceState::Up {
            None
        } else if self.director != ServiceState::Up || self.comfy != ServiceState::Up {
            Some("Start Chronos Foundry (Director :7777 + Comfy :8000) for council images.")
        } else if self.ollama != ServiceState::Up {
            Some("Start Ollama so the council can speak.")
        } else {
            None
        }
    }
}

pub fn probe_readiness() -> ReadinessSnapshot {
    ReadinessSnapshot {
        ollama: probe_ollama(),
        director: probe_director(),
        comfy: probe_comfy(),
    }
}

fn probe_ollama() -> ServiceState {
    match ureq::get("http://127.0.0.1:11434/api/tags")
        .timeout(Duration::from_millis(800))
        .call()
    {
        Ok(_) => ServiceState::Up,
        Err(_) => ServiceState::Down,
    }
}

fn probe_director() -> ServiceState {
    match ureq::get(&format!("{DIRECTOR_URL}/api/v1/status"))
        .timeout(Duration::from_millis(900))
        .call()
    {
        Ok(response) => match response.into_json::<Value>() {
            Ok(body) if body.get("readiness").and_then(Value::as_str) == Some("ready") => {
                ServiceState::Up
            }
            _ => ServiceState::Down,
        },
        Err(_) => ServiceState::Down,
    }
}

fn probe_comfy() -> ServiceState {
    match ureq::get(&format!("{}/system_stats", comfyui_url()))
        .timeout(Duration::from_millis(900))
        .call()
    {
        Ok(_) => ServiceState::Up,
        Err(_) => ServiceState::Down,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_line_names_all_three_services() {
        let snap = ReadinessSnapshot {
            ollama: ServiceState::Up,
            director: ServiceState::Down,
            comfy: ServiceState::Unknown,
        };
        let line = snap.banner_line();
        assert!(line.contains("Ollama up"));
        assert!(line.contains("Chronos Director down"));
        assert!(line.contains("Comfy unknown"));
        assert_eq!(
            snap.player_hint(),
            Some("Start Chronos Foundry (Director :7777 + Comfy :8000) for council images.")
        );
    }
}
