use cyconetic_ledger::{
    NeurorightsBoundPromptEnvelope, NeuralRope, NeuralRopeId, RopeBlock, RopeSeqNo, RopeStepKind,
    KerVector, EcoFootprint, SwarmTaskNodeId, CookbookPlaybookId, Did, AlnShardRef,
    BostromAddress, NeurorightsEnvelopeRef, GovernanceTriplet,
};
use cyconetic_ledger::hexstamp::{compute_hex_stamp_v1, HexStampInputV1};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CookbookVerb {
    Retrieve,
    Plan,
    Snapshot,
    ProposePage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptEnvelopeSurface {
    pub raw_prompt: String,
    pub verb: CookbookVerb,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeurorightsPromptEnvelope {
    pub surface: PromptEnvelopeSurface,

    pub did: Did,
    pub aln_context: AlnShardRef,
    pub bostrom_addr: BostromAddress,
    pub neurorights_profile: NeurorightsEnvelopeRef,

    pub roh_ceiling: f32,
}

pub trait SwarmRouter {
    fn route(
        &self,
        rope: &mut NeuralRope,
        prev_hex: &[u8; 8],
        seq_no: RopeSeqNo,
        env: NeurorightsPromptEnvelope,
        ker: KerVector,
        eco: EcoFootprint,
        swarm_step: SwarmTaskNodeId,
        playbook: CookbookPlaybookId,
        governance: GovernanceTriplet,
        step_kind: RopeStepKind,
    ) -> Result<[u8; 8], String>;
}

pub struct NeurorightsFirewallRouter<R> {
    inner: R,
}

impl<R> NeurorightsFirewallRouter<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R> SwarmRouter for NeurorightsFirewallRouter<R>
where
    R: SwarmRouter,
{
    fn route(
        &self,
        rope: &mut NeuralRope,
        prev_hex: &[u8; 8],
        seq_no: RopeSeqNo,
        env: NeurorightsPromptEnvelope,
        ker: KerVector,
        eco: EcoFootprint,
        swarm_step: SwarmTaskNodeId,
        playbook: CookbookPlaybookId,
        governance: GovernanceTriplet,
        step_kind: RopeStepKind,
    ) -> Result<[u8; 8], String> {
        // Enforce RoH ceiling at router level (hard ceiling).
        if ker.risk_of_harm > env.roh_ceiling {
            return Err(format!(
                "Blocked by neurorights firewall: RoH {} > ceiling {}",
                ker.risk_of_harm, env.roh_ceiling
            ));
        }

        self.inner.route(
            rope,
            prev_hex,
            seq_no,
            env,
            ker,
            eco,
            swarm_step,
            playbook,
            governance,
            step_kind,
        )
    }
}

// Example "ledger-emitting" router implementation.
pub struct LedgerRouter;

impl SwarmRouter for LedgerRouter {
    fn route(
        &self,
        rope: &mut NeuralRope,
        prev_hex: &[u8; 8],
        seq_no: RopeSeqNo,
        env: NeurorightsPromptEnvelope,
        ker: KerVector,
        eco: EcoFootprint,
        swarm_step: SwarmTaskNodeId,
        playbook: CookbookPlaybookId,
        governance: GovernanceTriplet,
        step_kind: RopeStepKind,
    ) -> Result<[u8; 8], String> {
        // Canonicalize core envelope for hex-stamp.
        #[derive(Serialize)]
        struct Canonical<'a> {
            surface: &'a PromptEnvelopeSurface,
            did: &'a Did,
            aln: &'a AlnShardRef,
            bostrom: &'a BostromAddress,
        }

        let canonical = Canonical {
            surface: &env.surface,
            did: &env.did,
            aln: &env.aln_context,
            bostrom: &env.bostrom_addr,
        };

        let canonical_bytes = bincode::serialize(&canonical)
            .map_err(|e| format!("Canonicalization error: {e}"))?;

        let input = HexStampInputV1 {
            prev_hex_stamp: *prev_hex,
            rope_seq_no: seq_no,
            canonical_envelope_bytes: &canonical_bytes,
            ker: &ker,
            governance: &governance,
        };

        let this_hex = compute_hex_stamp_v1(&input);

        let block = NeurorightsBoundPromptEnvelope {
            rope_id: rope.id.clone(),
            rope_seq_no: seq_no,
            did: env.did,
            aln_context: env.aln_context,
            bostrom_addr: env.bostrom_addr,
            neurorights_profile: env.neurorights_profile,
            governance,
            ker,
            swarmnet_step: swarm_step,
            cookbook_playbook: playbook,
            eco_impact: eco,
            step_kind,
            prev_hex_stamp: *prev_hex,
            this_hex_stamp: this_hex,
        };

        rope.blocks.push(RopeBlock::PromptHop(block));
        Ok(this_hex)
    }
}
