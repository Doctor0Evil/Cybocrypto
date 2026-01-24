use cybocrypto_aln_core::BioscaleSafe;
use serde::{Deserialize, Serialize};

/// Minimal facets for gaming/XR-focused neuro identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceFacet {
    pub role: String,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioscaleFacet {
    pub organic_cpu: bool,
    pub interface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrProfileFacet {
    pub avatar_id: String,
    pub world_realm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosurePolicy {
    pub minimal_disclosure: bool,
    pub revocable: bool,
    pub quantum_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroIdentity {
    pub bostrom_id: String,
    pub governance: GovernanceFacet,
    pub bioscale: BioscaleFacet,
    pub xr_profile: XrProfileFacet,
    pub disclosure: DisclosurePolicy,
}

/// Marker type for gaming/XR usage, also bioscale-safe (e.g. non-invasive).
#[derive(Debug, Clone)]
pub struct GamingNeuroIdentity;

impl BioscaleSafe for GamingNeuroIdentity {}

/// Macro to declare a neuro identity for gaming/XR.
#[macro_export]
macro_rules! neuro_identity {
    (
        id: $id:expr,
        facets: {
            governance: { role: $g_role:expr, level: $g_level:expr },
            bioscale: { organic_cpu: $b_ocpu:expr, interface: $b_iface:expr },
            xr_profile: { avatar_id: $x_avatar:expr, world_realm: $x_realm:expr }
        },
        constraints: [MinimalDisclosure, Revocable, QuantumReady]
    ) => {{
        $crate::NeuroIdentity {
            bostrom_id: $id.to_string(),
            governance: $crate::GovernanceFacet {
                role: $g_role.to_string(),
                level: $g_level,
            },
            bioscale: $crate::BioscaleFacet {
                organic_cpu: $b_ocpu,
                interface: $b_iface.to_string(),
            },
            xr_profile: $crate::XrProfileFacet {
                avatar_id: $x_avatar.to_string(),
                world_realm: $x_realm.to_string(),
            },
            disclosure: $crate::DisclosurePolicy {
                minimal_disclosure: true,
                revocable: true,
                quantum_ready: true,
            },
        }
    }};
}
