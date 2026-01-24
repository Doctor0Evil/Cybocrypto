use crate::replay::ReplayDescriptor;
use crate::frame_store::StoredFrameRecord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayEncodingFormat {
    JsonPretty,
    JsonCompact,
    BinaryBincode,
}

pub fn encode_replay(
    desc: &ReplayDescriptor,
    frames: &[StoredFrameRecord],
    format: ReplayEncodingFormat,
) -> Result<Vec<u8>, String> {
    #[derive(Serialize)]
    struct Bundle<'a> {
        descriptor: &'a ReplayDescriptor,
        frames: &'a [StoredFrameRecord],
    }

    let bundle = Bundle { descriptor: desc, frames };

    match format {
        ReplayEncodingFormat::JsonPretty => {
            serde_json::to_vec_pretty(&bundle).map_err(|e| e.to_string())
        }
        ReplayEncodingFormat::JsonCompact => {
            serde_json::to_vec(&bundle).map_err(|e| e.to_string())
        }
        ReplayEncodingFormat::BinaryBincode => {
            bincode::serialize(&bundle).map_err(|e| e.to_string())
        }
    }
}
