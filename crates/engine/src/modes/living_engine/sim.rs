//! Stylized pseudo-physics for the Living Engine. No Bevy types.

use crate::theme::Archetype;

pub const PROTOTYPE_SPHERES: [Archetype; 3] =
    [Archetype::Architect, Archetype::Sentinel, Archetype::Oracle];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resonance {
    pub angular_velocity: f32,
    pub radial_distance: f32,
    pub phase: f32,
    pub tilt_bias: f32,
    pub energetic_appetite: f32,
}

impl Resonance {
    pub fn for_archetype(archetype: Archetype) -> Self {
        match archetype {
            Archetype::Architect => Self {
                angular_velocity: 0.55,
                radial_distance: 4.0,
                phase: 0.0,
                tilt_bias: 0.08,
                energetic_appetite: 0.42,
            },
            Archetype::Sentinel => Self {
                angular_velocity: 0.38,
                radial_distance: 5.2,
                phase: 2.1,
                tilt_bias: 0.0,
                energetic_appetite: 0.28,
            },
            Archetype::Oracle => Self {
                angular_velocity: 0.72,
                radial_distance: 6.1,
                phase: 4.2,
                tilt_bias: 0.18,
                energetic_appetite: 0.61,
            },
            _ => Self {
                angular_velocity: 0.5,
                radial_distance: 4.5,
                phase: 0.0,
                tilt_bias: 0.0,
                energetic_appetite: 0.4,
            },
        }
    }

    pub fn clamped(mut self) -> Self {
        self.angular_velocity = self.angular_velocity.clamp(0.08, 1.8);
        self.radial_distance = self.radial_distance.clamp(2.4, 8.0);
        self.tilt_bias = self.tilt_bias.clamp(-0.4, 0.4);
        self.energetic_appetite = self.energetic_appetite.clamp(0.05, 1.0);
        self.phase = self.phase.rem_euclid(std::f32::consts::TAU);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AuraLayers {
    pub local: f32,
    pub reservoir: f32,
    pub ambient: f32,
}

impl AuraLayers {
    pub fn resting() -> Self {
        Self {
            local: 0.55,
            reservoir: 0.62,
            ambient: 0.48,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Harmony {
    pub coherence: f32,
    pub metabolic_strain: f32,
    pub asymmetry: f32,
    pub anomaly_pressure: f32,
}

impl Harmony {
    pub fn score(self) -> f32 {
        (self.coherence * 1.2 - self.metabolic_strain - self.asymmetry * 0.6 - self.anomaly_pressure)
            .clamp(0.0, 1.0)
    }

    pub fn overloaded(self) -> bool {
        self.metabolic_strain > 0.82 || self.aura_overload_proxy()
    }

    fn aura_overload_proxy(self) -> bool {
        self.coherence < 0.12 && self.metabolic_strain > 0.7
    }

    pub fn starving(self) -> bool {
        self.coherence < 0.18 && self.metabolic_strain < 0.22
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplantSlot {
    Pathway,
    Reservoir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplantKind {
    Dampen,
    Charge,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Implant {
    pub slot: ImplantSlot,
    pub kind: ImplantKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VirenStrain {
    pub infected_index: Option<usize>,
    pub intensity: f32,
}

impl VirenStrain {
    pub fn is_active(self) -> bool {
        self.infected_index.is_some() && self.intensity > 0.02
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LivingSim {
    pub spheres: [Resonance; 3],
    pub aura: AuraLayers,
    pub viren: VirenStrain,
    pub implants: [Option<Implant>; 2],
    pub selected: usize,
    pub elapsed: f32,
}

impl Default for LivingSim {
    fn default() -> Self {
        Self {
            spheres: [
                Resonance::for_archetype(Archetype::Architect),
                Resonance::for_archetype(Archetype::Sentinel),
                Resonance::for_archetype(Archetype::Oracle),
            ],
            aura: AuraLayers::resting(),
            viren: VirenStrain::default(),
            implants: [None, None],
            selected: 0,
            elapsed: 0.0,
        }
    }
}

impl LivingSim {
    pub fn breath(&mut self, dt: f32) -> Harmony {
        self.elapsed += dt;
        for sphere in &mut self.spheres {
            sphere.phase += sphere.angular_velocity * dt;
            *sphere = sphere.clamped();
        }

        let mean_radius: f32 =
            self.spheres.iter().map(|s| s.radial_distance).sum::<f32>() / 3.0;
        let mean_omega: f32 =
            self.spheres.iter().map(|s| s.angular_velocity).sum::<f32>() / 3.0;
        let radius_var = self
            .spheres
            .iter()
            .map(|s| (s.radial_distance - mean_radius).abs())
            .sum::<f32>()
            / 3.0;
        let omega_var = self
            .spheres
            .iter()
            .map(|s| (s.angular_velocity - mean_omega).abs())
            .sum::<f32>()
            / 3.0;

        let appetite: f32 = self
            .spheres
            .iter()
            .map(|s| s.energetic_appetite)
            .sum::<f32>()
            / 3.0;

        let mut local_draw = appetite * 0.35 * dt;
        let mut reservoir_draw = mean_omega * 0.12 * dt;
        for implant in self.implants.iter().flatten() {
            match (implant.slot, implant.kind) {
                (ImplantSlot::Pathway, ImplantKind::Dampen) => {
                    reservoir_draw *= 0.72;
                    for sphere in &mut self.spheres {
                        sphere.angular_velocity *= 0.999;
                    }
                }
                (ImplantSlot::Pathway, ImplantKind::Charge) => {
                    local_draw *= 1.15;
                    for sphere in &mut self.spheres {
                        sphere.energetic_appetite = (sphere.energetic_appetite + 0.08 * dt).min(1.0);
                    }
                }
                (ImplantSlot::Reservoir, ImplantKind::Dampen) => {
                    self.aura.reservoir = (self.aura.reservoir + 0.05 * dt).min(1.0);
                }
                (ImplantSlot::Reservoir, ImplantKind::Charge) => {
                    self.aura.reservoir = (self.aura.reservoir + 0.12 * dt).min(1.0);
                    reservoir_draw *= 1.1;
                }
            }
        }

        self.aura.local = (self.aura.local - local_draw + 0.08 * dt).clamp(0.0, 1.0);
        self.aura.reservoir = (self.aura.reservoir - reservoir_draw).clamp(0.0, 1.0);
        self.aura.ambient = ((self.aura.local + self.aura.reservoir) * 0.5).clamp(0.0, 1.0);

        if self.viren.is_active() {
            if let Some(index) = self.viren.infected_index {
                self.viren.intensity = (self.viren.intensity + 0.12 * dt).min(1.0);
                if let Some(sphere) = self.spheres.get_mut(index) {
                    sphere.angular_velocity = (sphere.angular_velocity + 0.35 * dt).min(1.8);
                    sphere.energetic_appetite = (sphere.energetic_appetite + 0.2 * dt).min(1.0);
                }
                self.aura.local = (self.aura.local - 0.18 * dt).max(0.0);
            }
        }

        let coherence = (1.0 - radius_var / 4.0 - omega_var / 1.2).clamp(0.0, 1.0)
            * self.aura.ambient.clamp(0.15, 1.0);
        let metabolic_strain = ((mean_omega - 0.4).max(0.0) * 0.7
            + (1.0 - self.aura.reservoir) * 0.55
            + self.viren.intensity * 0.4)
            .clamp(0.0, 1.0);
        let asymmetry = (radius_var / 3.5 + omega_var / 0.9).clamp(0.0, 1.0);
        let anomaly_pressure = if self.viren.is_active() {
            (0.35 + self.viren.intensity * 0.65).clamp(0.0, 1.0)
        } else {
            (omega_var * 0.4).clamp(0.0, 1.0)
        };

        Harmony {
            coherence,
            metabolic_strain,
            asymmetry,
            anomaly_pressure,
        }
    }

    pub fn tune_selected_omega(&mut self, delta: f32) {
        if let Some(sphere) = self.spheres.get_mut(self.selected) {
            sphere.angular_velocity += delta;
            *sphere = sphere.clamped();
        }
    }

    pub fn tune_selected_radius(&mut self, delta: f32) {
        if let Some(sphere) = self.spheres.get_mut(self.selected) {
            sphere.radial_distance += delta;
            *sphere = sphere.clamped();
        }
    }

    pub fn cycle_selected(&mut self) {
        self.selected = (self.selected + 1) % self.spheres.len();
    }

    pub fn toggle_implant(&mut self, slot: ImplantSlot, kind: ImplantKind) {
        let index = match slot {
            ImplantSlot::Pathway => 0,
            ImplantSlot::Reservoir => 1,
        };
        let next = Implant { slot, kind };
        self.implants[index] = match self.implants[index] {
            Some(current) if current == next => None,
            _ => Some(next),
        };
    }

    pub fn infect_hottest(&mut self) {
        if self.viren.is_active() {
            return;
        }
        let index = self
            .spheres
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.energetic_appetite
                    .total_cmp(&b.energetic_appetite)
            })
            .map(|(index, _)| index)
            .unwrap_or(0);
        self.viren = VirenStrain {
            infected_index: Some(index),
            intensity: 0.18,
        };
    }

    /// Ember Covenant cure: counter-frequency when the selected sphere's phase
    /// is near zero (the struck E-flat window).
    pub fn try_cure(&mut self) -> bool {
        let Some(index) = self.viren.infected_index else {
            return false;
        };
        if self.selected != index {
            return false;
        }
        let phase = self.spheres[index].phase.rem_euclid(std::f32::consts::TAU);
        let window = 0.55;
        let aligned = phase < window || phase > std::f32::consts::TAU - window;
        if aligned {
            self.viren = VirenStrain::default();
            if let Some(sphere) = self.spheres.get_mut(index) {
                sphere.angular_velocity = Resonance::for_archetype(PROTOTYPE_SPHERES[index])
                    .angular_velocity;
            }
            true
        } else {
            false
        }
    }

    pub fn position(&self, index: usize) -> [f32; 3] {
        let sphere = self.spheres[index];
        let x = sphere.radial_distance * sphere.phase.cos();
        let z = sphere.radial_distance * sphere.phase.sin();
        let y = 1.6 + sphere.tilt_bias * 2.0 + (sphere.phase * 2.0).sin() * 0.2;
        [x, y, z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_sim_has_three_prototype_spheres() {
        let sim = LivingSim::default();
        assert_eq!(sim.spheres.len(), 3);
        assert_eq!(PROTOTYPE_SPHERES.len(), 3);
    }

    #[test]
    fn breath_is_finite_and_bounded() {
        let mut sim = LivingSim::default();
        let harmony = sim.breath(0.5);
        assert!(harmony.coherence.is_finite());
        assert!((0.0..=1.0).contains(&harmony.coherence));
        assert!((0.0..=1.0).contains(&harmony.metabolic_strain));
        assert!((0.0..=1.0).contains(&sim.aura.reservoir));
    }

    #[test]
    fn viren_raises_anomaly_and_cure_requires_phase_window() {
        let mut sim = LivingSim::default();
        sim.infect_hottest();
        assert!(sim.viren.is_active());
        let before = sim.breath(0.4);
        assert!(before.anomaly_pressure > 0.3);
        sim.selected = sim.viren.infected_index.unwrap();
        sim.spheres[sim.selected].phase = 0.05;
        assert!(sim.try_cure());
        assert!(!sim.viren.is_active());
    }

    #[test]
    fn implants_toggle_and_dampen_reduces_strain_vs_uncontrolled_spin() {
        let mut wild = LivingSim::default();
        for sphere in &mut wild.spheres {
            sphere.angular_velocity = 1.7;
        }
        let wild_strain = wild.breath(1.0).metabolic_strain;

        let mut damped = LivingSim::default();
        for sphere in &mut damped.spheres {
            sphere.angular_velocity = 1.7;
        }
        damped.toggle_implant(ImplantSlot::Pathway, ImplantKind::Dampen);
        let damped_strain = damped.breath(1.0).metabolic_strain;
        assert!(damped_strain <= wild_strain);
    }

    #[test]
    fn starve_and_overload_flags_are_honest() {
        let starving = Harmony {
            coherence: 0.1,
            metabolic_strain: 0.1,
            asymmetry: 0.0,
            anomaly_pressure: 0.0,
        };
        assert!(starving.starving());
        assert!(!starving.overloaded());
        let overloaded = Harmony {
            coherence: 0.4,
            metabolic_strain: 0.9,
            asymmetry: 0.2,
            anomaly_pressure: 0.1,
        };
        assert!(overloaded.overloaded());
    }
}
