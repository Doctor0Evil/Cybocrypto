use serde::{Deserialize, Serialize};

/// Node kinds in the policy graph aligned with your design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Right,
    Role,
    Context,
    Constraint,
    Remedy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeKind {
    Requires,
    Forbids,
    Prioritizes,
    FallbackTo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyGraphAst {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
