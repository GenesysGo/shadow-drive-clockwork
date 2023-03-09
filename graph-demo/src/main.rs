use graph_demo::*;
use std::io::Write;

fn main() {
    let nodes: Vec<GraphNode> = vec![
        GraphNode {
            name: "Alice".to_string(),
            next: "Bob".to_string(),
        },
        GraphNode {
            name: "Bob".to_string(),
            next: "Carol".to_string(),
        },
        GraphNode {
            name: "Carol".to_string(),
            next: "Dave".to_string(),
        },
        GraphNode {
            name: "Dave".to_string(),
            next: "Alice".to_string(),
        },
    ];

    const DIR: &str = "nodes";
    std::fs::create_dir_all(DIR).unwrap();
    for node in nodes {
        let node_bytes = rkyv::to_bytes::<_, 256>(&node).unwrap();
        let mut node_file = std::fs::File::create(format!("{DIR}/{}", &node.name)).unwrap();
        node_file.write_all(&node_bytes).unwrap();
    }
}
