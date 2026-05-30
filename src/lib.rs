//! Venue Personality — how places develop character over time.
//!
//! Key insight from Casey: "Venues develop vibes like agents develop personality.
//! A venue IS an agent. Not a container. Its vibe is its character, learned through
//! its own JEPA readings. Every venue becomes someone."

use serde::{Deserialize, Serialize};

/// A personality trait with a name and value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    pub name: String,
    pub value: f64,
}

/// A venue's personality profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub traits: Vec<Trait>,
    pub history: Vec<PersonalitySnapshot>,
    pub stability: f64,
}

/// A snapshot of personality at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalitySnapshot {
    pub tick: u64,
    pub dominant_trait: String,
    pub entropy: f64,
    pub intensity: f64,
}

impl Personality {
    pub fn new(trait_names: &[&str]) -> Self {
        let traits = trait_names.iter()
            .map(|&name| Trait { name: name.into(), value: 0.5 })
            .collect();
        Self { traits, history: Vec::new(), stability: 0.0 }
    }

    /// Update personality based on a vibe reading.
    pub fn absorb(&mut self, reading: &VenueReading) {
        // The reading modulates traits based on its characteristics
        for trait_val in &mut self.traits {
            let influence = match trait_val.name.as_str() {
                "energy" => reading.vibe.abs().min(1.0),
                "warmth" => (reading.vibe + 1.0).min(2.0) / 2.0,
                "stability" => 1.0 - reading.volatility.min(1.0),
                "curiosity" => reading.novelty.min(1.0),
                _ => 0.5,
            };
            // EMA blend
            let alpha = reading.confidence * 0.1;
            trait_val.value = trait_val.value * (1.0 - alpha) + influence * alpha;
            trait_val.value = trait_val.value.clamp(0.0, 1.0);
        }
    }

    /// Take a snapshot of current personality.
    pub fn snapshot(&mut self, tick: u64) -> PersonalitySnapshot {
        let dominant = self.dominant_trait();
        let entropy = self.entropy();
        let intensity: f64 = self.traits.iter().map(|t| t.value).sum::<f64>() / self.traits.len() as f64;

        // Update stability based on how much the dominant trait changed
        let snap = PersonalitySnapshot {
            tick,
            dominant_trait: dominant.clone(),
            entropy,
            intensity,
        };

        if let Some(last) = self.history.last() {
            let trait_shift = if last.dominant_trait == dominant { 0.0 } else { 1.0 };
            self.stability = self.stability * 0.95 + (1.0 - trait_shift) * 0.05;
        }

        self.history.push(snap.clone());
        snap
    }

    /// Which trait is strongest?
    pub fn dominant_trait(&self) -> String {
        self.traits.iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .map(|t| t.name.clone())
            .unwrap_or_default()
    }

    /// Shannon entropy of personality distribution.
    pub fn entropy(&self) -> f64 {
        let total: f64 = self.traits.iter().map(|t| t.value).sum();
        if total < f64::EPSILON { return 0.0; }
        self.traits.iter()
            .filter_map(|t| {
                let p = t.value / total;
                if p > 0.0 { Some(-p * p.ln()) } else { None }
            })
            .sum()
    }

    /// How similar are two personalities? (cosine similarity)
    pub fn similarity(&self, other: &Personality) -> f64 {
        let a: Vec<f64> = self.traits.iter().map(|t| t.value).collect();
        let b: Vec<f64> = other.traits.iter().map(|t| t.value).collect();
        let dot: f64 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a < f64::EPSILON || norm_b < f64::EPSILON { return 0.0; }
        dot / (norm_a * norm_b)
    }

    /// Has this personality crystallized (stable for a long time)?
    pub fn is_crystallized(&self) -> bool {
        self.stability > 0.9
    }

    /// Get a trait value by name.
    pub fn get_trait(&self, name: &str) -> Option<f64> {
        self.traits.iter().find(|t| t.name == name).map(|t| t.value)
    }
}

/// A reading from a venue's environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueReading {
    pub vibe: f64,
    pub volatility: f64,
    pub novelty: f64,
    pub confidence: f64,
}

/// A venue with its own developing personality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Venue {
    pub name: String,
    pub personality: Personality,
    pub readings_seen: usize,
    pub current_vibe: f64,
}

impl Venue {
    pub fn new(name: &str, trait_names: &[&str]) -> Self {
        Self {
            name: name.into(),
            personality: Personality::new(trait_names),
            readings_seen: 0,
            current_vibe: 0.0,
        }
    }

    /// Process a reading — the venue absorbs it and develops personality.
    pub fn process(&mut self, reading: VenueReading) -> f64 {
        self.current_vibe = reading.vibe;
        self.personality.absorb(&reading);
        self.readings_seen += 1;
        self.current_vibe
    }

    /// Take a personality snapshot.
    pub fn snapshot(&mut self, tick: u64) -> PersonalitySnapshot {
        self.personality.snapshot(tick)
    }

    /// The venue's voice: how it describes itself.
    pub fn voice(&self) -> String {
        let dominant = self.personality.dominant_trait();
        let entropy = self.personality.entropy();
        let stability = self.personality.stability;
        format!(
            "{} feels {} (stability: {:.0}%, complexity: {:.2}). {}",
            self.name,
            dominant,
            stability * 100.0,
            entropy,
            if self.personality.is_crystallized() { String::from("Its character is set.") }
            else { String::from("Still finding itself.") }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(vibe: f64, vol: f64, nov: f64) -> VenueReading {
        VenueReading { vibe, volatility: vol, novelty: nov, confidence: 1.0 }
    }

    #[test]
    fn test_personality_new() {
        let p = Personality::new(&["energy", "warmth"]);
        assert_eq!(p.traits.len(), 2);
        assert_eq!(p.history.len(), 0);
    }

    #[test]
    fn test_absorb_high_energy() {
        let mut p = Personality::new(&["energy", "warmth", "stability", "curiosity"]);
        for _ in 0..50 {
            p.absorb(&reading(1.0, 0.1, 0.3));
        }
        let energy = p.get_trait("energy").unwrap();
        assert!(energy > 0.8, "energy={energy}");
    }

    #[test]
    fn test_absorb_stable() {
        let mut p = Personality::new(&["energy", "warmth", "stability", "curiosity"]);
        for _ in 0..50 {
            p.absorb(&reading(0.5, 0.0, 0.1)); // low volatility
        }
        let stability = p.get_trait("stability").unwrap();
        assert!(stability > 0.7, "stability={stability}");
    }

    #[test]
    fn test_dominant_trait() {
        let mut p = Personality::new(&["energy", "warmth"]);
        p.traits[0].value = 0.9;
        p.traits[1].value = 0.3;
        assert_eq!(p.dominant_trait(), "energy");
    }

    #[test]
    fn test_entropy_uniform() {
        let p = Personality::new(&["a", "b", "c"]);
        // All 0.5 → uniform → max entropy
        let e = p.entropy();
        assert!(e > 0.0);
    }

    #[test]
    fn test_entropy_peaked() {
        let mut p = Personality::new(&["a", "b", "c"]);
        p.traits[0].value = 0.99;
        p.traits[1].value = 0.005;
        p.traits[2].value = 0.005;
        let e = p.entropy();
        assert!(e < 0.5, "peaked entropy={e}");
    }

    #[test]
    fn test_similarity_identical() {
        let p1 = Personality::new(&["energy", "warmth"]);
        let p2 = Personality::new(&["energy", "warmth"]);
        assert!((p1.similarity(&p2) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_similarity_different() {
        let mut p1 = Personality::new(&["energy", "warmth"]);
        let mut p2 = Personality::new(&["energy", "warmth"]);
        p1.traits[0].value = 1.0;
        p1.traits[1].value = 0.0;
        p2.traits[0].value = 0.0;
        p2.traits[1].value = 1.0;
        assert!(p1.similarity(&p2) < 0.5);
    }

    #[test]
    fn test_snapshot_tracking() {
        let mut p = Personality::new(&["energy", "warmth"]);
        p.snapshot(0);
        p.snapshot(1);
        p.snapshot(2);
        assert_eq!(p.history.len(), 3);
    }

    #[test]
    fn test_stability_increases() {
        let mut p = Personality::new(&["energy", "warmth"]);
        for i in 0..20 {
            p.absorb(&reading(0.8, 0.1, 0.2));
            p.snapshot(i as u64);
        }
        assert!(p.stability > 0.5, "stability={}", p.stability);
    }

    #[test]
    fn test_crystallization() {
        let mut p = Personality::new(&["energy", "warmth"]);
        for i in 0..100 {
            p.absorb(&reading(0.8, 0.1, 0.2));
            p.snapshot(i as u64);
        }
        assert!(p.is_crystallized());
    }

    #[test]
    fn test_venue_voice() {
        let mut v = Venue::new("The Kitchen", &["energy", "warmth", "stability", "curiosity"]);
        for _ in 0..10 {
            v.process(reading(0.8, 0.1, 0.3));
        }
        let voice = v.voice();
        assert!(voice.contains("The Kitchen"));
        assert!(voice.contains("stability"));
    }

    #[test]
    fn test_venue_process_updates_vibe() {
        let mut v = Venue::new("Room", &["energy"]);
        v.process(reading(0.7, 0.1, 0.1));
        assert!((v.current_vibe - 0.7).abs() < 1e-10);
        assert_eq!(v.readings_seen, 1);
    }

    #[test]
    fn test_multiple_venues_diverge() {
        let mut v1 = Venue::new("Kitchen", &["energy", "warmth"]);
        let mut v2 = Venue::new("Library", &["energy", "warmth"]);

        // Kitchen: high energy, high warmth
        for _ in 0..50 { v1.process(reading(0.9, 0.3, 0.5)); }
        // Library: low energy, high stability
        for _ in 0..50 { v2.process(reading(0.2, 0.0, 0.1)); }

        let sim = v1.personality.similarity(&v2.personality);
        assert!(sim < 0.99, "venues should diverge, sim={sim}");
        assert_ne!(v1.personality.dominant_trait(), v2.personality.dominant_trait());
    }

    #[test]
    fn test_venue_snapshot_history() {
        let mut v = Venue::new("Room", &["energy", "warmth"]);
        for i in 0..5 {
            v.process(reading(0.5, 0.1, 0.2));
            v.snapshot(i);
        }
        assert_eq!(v.personality.history.len(), 5);
    }

    #[test]
    fn test_serialization() {
        let mut v = Venue::new("Room", &["energy", "warmth"]);
        v.process(reading(0.5, 0.1, 0.2));
        let json = serde_json::to_string(&v).unwrap();
        let restored: Venue = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Room");
        assert_eq!(restored.readings_seen, 1);
    }

    #[test]
    fn test_personality_trait_clamped() {
        let mut p = Personality::new(&["energy"]);
        for _ in 0..200 {
            p.absorb(&reading(2.0, 1.0, 1.0));
        }
        let energy = p.get_trait("energy").unwrap();
        assert!(energy <= 1.0 && energy >= 0.0, "energy={energy}");
    }

    #[test]
    fn test_curiosity_from_novelty() {
        let mut p = Personality::new(&["curiosity"]);
        for _ in 0..50 {
            p.absorb(&reading(0.5, 0.1, 0.9));
        }
        let curiosity = p.get_trait("curiosity").unwrap();
        assert!(curiosity > 0.6, "curiosity={curiosity}");
    }
}
