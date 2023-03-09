#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct GraphNode {
    pub name: String,
    pub next: String,
}
