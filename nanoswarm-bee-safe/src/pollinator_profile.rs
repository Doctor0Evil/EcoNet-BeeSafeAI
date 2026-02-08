use serde::{Serialize, Deserialize};

/// Attachment behavior near biological surfaces.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BioAttachmentMode {
    /// Absolute prohibition on any binding or deliberate contact with pollinators.
    NoBindPollinator,
    /// Future: other modes for non-pollinator biology only.
    Neutral,
}

/// Governance-level consent states for any object the swarm can target.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsentLevel {
    /// Default for inanimate environment.
    Unspecified,
    /// Entities that can be affected under normal host-governance.
    HostConsenting,
    /// Honeybee / pollinator objects – non‑negotiable protection.
    PollinatorProtected,
}

/// Static, per-object profile for honeybees and other protected pollinators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollinatorProfile {
    /// Explicit, machine-checkable protection flag – must always be true for bees.
    pub is_protected: bool,

    /// Attachment mode – must be `NoBindPollinator` for bees.
    pub bio_affinity_mode: BioAttachmentMode,

    /// Maximum allowed local power density (W/m^3) when this object is inside the node's field.
    pub energy_budget: f32,

    /// Maximum allowed EM emission envelope (W/m^2 or V/m) in bee-relevant bands.
    pub emission_limit: f32,

    /// Non-overrideable consent level – pinned to `PollinatorProtected` for honeybees.
    pub consent_state: ConsentLevel,
}

impl PollinatorProfile {
    pub fn new_bee_default() -> Self {
        Self {
            is_protected: true,
            bio_affinity_mode: BioAttachmentMode::NoBindPollinator,
            energy_budget: 0.0, // “negligible” – refined from BeeNeuralCorridor rows.
            emission_limit: 0.0,
            consent_state: ConsentLevel::PollinatorProtected,
        }
    }

    /// Hard invariant: a bee profile can never be downgraded by ordinary code paths.
    pub fn assert_invariants(&self) -> bool {
        self.is_protected
            && matches!(self.bio_affinity_mode, BioAttachmentMode::NoBindPollinator)
            && matches!(self.consent_state, ConsentLevel::PollinatorProtected)
    }
}
