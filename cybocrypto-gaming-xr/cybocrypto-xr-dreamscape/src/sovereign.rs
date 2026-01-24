use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereignRegion {
    Eu,
    Us,
    Latam,
    Apac,
    Custom(String), // e.g. "Offshore-Node-01"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackendKind {
    ObjectStore,
    RelationalDb,
    TimeSeriesDb,
    AppendLog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignStorageTarget {
    pub region: SovereignRegion,
    pub backend: StorageBackendKind,
    pub endpoint_label: String, // your own internal label, not GitHub
    pub bucket_or_db: String,
    pub collection: String,
}
