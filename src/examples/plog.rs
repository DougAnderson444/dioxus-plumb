//! Provenance Log Diagram
//!╔═════════════════════════[ Provenance Log and VLAD ]══════════════════════════╗
//! ║                                                                              ║
//1 ║  ╭────────────────────────[Distributed Hash Table]────────────────────────╮  ║
//! ║  │                                                                        │  ║
//! ║  │ ╭─[VLAD]──────┬──────────────╮                     ╭─[Mutable Value]─╮ │  ║
//! ║  │ │   <Sig of> ───> <WASM CID> │ ───── maps to ────> │      <CID>      │ │  ║
//! ║  │ ╰─────────────┴────────────┬─╯                     ╰─┬───────────────╯ │  ║
//! ║  │          ^                 │                         │                 │  ║
//! ║  ╰──────────│─────────────────│─────────────────────────│─────────────────╯  ║
//! ║             │  ╭─ references ─╯                         ╰ references ╮       ║
//! ║             │  │                                                     │       ║
//! ║             ╰───── verifies ──╮                                      │       ║
//! ║  ╭─────────────│──────────────│──────────────────────────────────────│────╮  ║
//! ║  │             v              │                                      v    │  ║
//! ║  │ ╭─[WASM]─────────╮         │      ╭─[Foot]────────╮  ╭─[Head]────────╮ │  ║
//! ║  │ │ (module        │         │      │ Seqno 0       │  │ Seqno 1       │ │  ║
//! ║  │ │   (func $main  │         │    X── Prev NULL     │<── Prev          │ │  ║
//! ║  │ │     return     │         ╰─────── Vlad Pubkey   │  │               │ │  ║
//! ║  │ │   )            │ ─ verifies ──> │               │  │               │ │  ║
//! ║  │ │ )              │                │               │  │               │ │  ║
//! ║  │ ╰────────────────╯                ╰───────────────╯  ╰───────────────╯ │  ║
//! ║  ╰───────────────────────[Content Addressable Storage]────────────────────╯  ║
//! ║                                                                              ║
//! ╚══════════════════════════════════════════════════════════════════════════════╝
use dioxus::prelude::*;
// Corrected import based on edge_arena.rs example and ls output
use dioxus_plumb::{edge_renderer::EdgeArena, graph_data::parse_edges};

// Enum to categorize the different types of nodes in the provenance log diagram
#[derive(Clone, Copy, Debug, PartialEq)]
enum PlogNodeType {
    Dht,
    Vlad,
    Wasm,
    MutableValue,
    Foot,
    Head,
    Cas,
}

// Component to render individual nodes with specific styling based on their type
#[component]
fn PlogNodeRenderer(id: String, node_type: PlogNodeType, label: String) -> Element {
    let mut base_classes = "border p-4 rounded-lg shadow-md w-48 text-center"; // Added fixed width and center text
    let mut text_color = "text-gray-800";
    let mut bg_color = "bg-white";

    // Apply different background and text colors based on the node type for visual distinction
    match node_type {
        PlogNodeType::Dht => {
            bg_color = "bg-blue-100";
            text_color = "text-blue-800";
        }
        PlogNodeType::Vlad => {
            bg_color = "bg-green-100";
            text_color = "text-green-800";
        }
        PlogNodeType::Wasm => {
            bg_color = "bg-yellow-100";
            text_color = "text-yellow-800";
        }
        PlogNodeType::MutableValue => {
            bg_color = "bg-purple-100";
            text_color = "text-purple-800";
        }
        PlogNodeType::Foot => {
            bg_color = "bg-red-100";
            text_color = "text-red-800";
        }
        PlogNodeType::Head => {
            bg_color = "bg-orange-100";
            text_color = "text-orange-800";
        }
        PlogNodeType::Cas => {
            bg_color = "bg-gray-200";
            text_color = "text-gray-800";
        }
    }

    let classes = format!("{} {} {}", base_classes, bg_color, text_color);

    rsx! {
        // The 'id' prop is crucial for EdgeArena to identify and position the node correctly.
        div {
            id: id.clone(),
            class: classes,
            // Display the label provided from the DOT definition.
            h3 { class: "text-lg font-bold mb-2", "{label}" }
        }
    }
}

/// Component to render the Provenance Log Diagram using EdgeArena.
#[component]
pub fn PlogDiagram() -> Element {
    // DOT graph definition describing the provenance log structure and relationships.
    // We will parse only the edges from this string using parse_edges.
    let dot_graph_edges_only = r#"
        digraph ProvenanceLogEdges {
            // Node IDs used in edges.
            "dht"; "vlad"; "wasm"; "mutable_value"; "foot"; "head"; "cas";

            // Edge definitions based on the diagram's arrows and labels, connecting the nodes.
            // VLAD -> <WASM CID> -> <Mutable Value>
            "vlad" -> "wasm" [label="<Sig of>\nmaps to\n<WASM CID>"]; // Combining labels for clarity on the relationship
            "wasm" -> "mutable_value" [label="maps to"]; // The WASM code concept maps to the Mutable Value concept

            // References and Verifies links from the diagram.
            "mutable_value" -> "cas" [label="references"]; // Mutable Value refers to content in CAS
            "wasm" -> "cas" [label="references"];         // WASM module refers to content in CAS

            "wasm" -> "foot" [label="verifies"];         // WASM module verifies the Foot entry
            "wasm" -> "head" [label="verifies"];         // WASM module verifies the Head entry

            "foot" -> "cas" [label="verifies"];         // Foot entry verifies content in CAS
            "head" -> "cas" [label="verifies"];         // Head entry verifies content in CAS

            // DHT related connections, indicating how DHT indexes or relates to other components.
            "dht" -> "vlad" [label="<WASM CID>"]; // DHT stores/indexes VLAD via WASM CID
            "dht" -> "cas" [label="<CID>"];     // DHT stores/indexes CAS via CID
        }
    "#;

    // Parse the DOT graph string to get only the edges.
    // EdgeArena expects a list of edges.
    let edges_data = parse_edges(dot_graph_edges_only).unwrap_or_default();

    // Manually define the nodes and their properties, as EdgeArena renders child components for nodes.
    // We extract this information from the original diagram's node definitions.
    let nodes_data = vec![
        ("dht", "Distributed Hash Table", PlogNodeType::Dht),
        ("vlad", "VLAD", PlogNodeType::Vlad),
        ("wasm", "WASM\n(module...)", PlogNodeType::Wasm),
        (
            "mutable_value",
            "Mutable Value\n<CID>",
            PlogNodeType::MutableValue,
        ),
        ("foot", "Foot\nSeqno 0", PlogNodeType::Foot),
        ("head", "Head\nSeqno 1", PlogNodeType::Head),
        ("cas", "Content Addressable Storage", PlogNodeType::Cas),
    ];

    rsx! {
        div {
            class: "mt-6 border border-gray-300 rounded-xl p-4",
            h2 { class: "text-xl font-bold mb-4", "Provenance Log Diagram Demo" }

            // EdgeArena is the component that renders the graph layout.
            EdgeArena {
                edges: edges_data, // Provide the parsed edges to EdgeArena.
                div { // Container for the nodes. EdgeArena will position these based on edge data.
                    class: "flex flex-wrap gap-8 justify-center", // Use flexbox for a responsive node layout.
                    // Iterate over the manually defined node data and render a PlogNodeRenderer for each.
                    // EdgeArena will match these components to the nodes specified in the edges by their 'id' prop.
                    {nodes_data.iter().map(|(id, label, node_type)| {
                        rsx! {
                            PlogNodeRenderer {
                                id: id.to_string(),       // Node ID required by EdgeArena.
                                node_type: *node_type,  // Type for styling.
                                label: label.to_string(), // Label to display within the node.
                            }
                        }
                    })}
                }
            }
        }
    }
}
