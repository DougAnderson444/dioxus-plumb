//! Provenance Log Diagram
//! ╔═════════════════════════[ Provenance Log and VLAD ]══════════════════════════╗
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
use dioxus_plumb::{
    edge_renderer::EdgeArena,
    graph_data::{parse_graph, GraphData},
};
use std::str::FromStr;

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
    Entries,
}

impl FromStr for PlogNodeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Dht" => Ok(PlogNodeType::Dht),
            "Vlad" => Ok(PlogNodeType::Vlad),
            "Wasm" => Ok(PlogNodeType::Wasm),
            "MutableValue" => Ok(PlogNodeType::MutableValue),
            "Foot" => Ok(PlogNodeType::Foot),
            "Head" => Ok(PlogNodeType::Head),
            "Cas" => Ok(PlogNodeType::Cas),
            "Entries" => Ok(PlogNodeType::Entries),
            _ => Err(()),
        }
    }
}

// Component to render individual nodes with specific styling based on their type
#[component]
fn PlogNodeRenderer(id: String, node_type: PlogNodeType, label: String) -> Element {
    let base_classes = "border p-4 rounded-lg shadow-md text-center"; // Removed w-48 for more flexible sizing
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
        PlogNodeType::Entries => {
            bg_color = "bg-purple-100";
            text_color = "text-purple-800";
        }
    }

    let classes = format!("{} {} {}", base_classes, bg_color, text_color);

    rsx! {
        // The 'id' prop is crucial for EdgeArena to identify and position the node correctly.
        div {
            id: id.clone(),
            class: classes,
            // Display the label provided from the DOT definition.
            h3 {
                class: "text-lg font-bold mb-2",
                style: "white-space: pre-wrap;", // Handle newlines in labels
                "{label}"
            }
        }
    }
}

#[component]
fn Graph(graph: GraphData) -> Element {
    let direction_class = graph.direction.flex_class();

    rsx! {
        div {
            class: format!("border rounded-lg p-4 relative {}", direction_class),
            if let Some(label) = &graph.label {
                h3 {
                    class: "absolute -top-3 left-1/2 -translate-x-1/2 bg-white px-2 text-lg font-bold",
                    "{label}"
                }
            }
            div {
                class: format!("flex {} gap-8 pt-4", direction_class),

                // Render nodes
                {graph.nodes.iter().map(|node| {
                    let node_type = PlogNodeType::from_str(node.label.as_deref().unwrap_or("")).unwrap_or(PlogNodeType::Cas);
                    rsx! {
                        PlogNodeRenderer {
                            id: node.id.clone(),
                            node_type: node_type,
                            label: node.label.clone().unwrap_or_default(),
                        }
                    }
                })}

                // Render subgraphs
                {graph.subgraphs.iter().map(|subgraph| {
                    rsx! {
                        Graph {
                            graph: subgraph.clone()
                        }
                    }
                })}
            }
        }
    }
}

/// Component to render the Provenance Log Diagram using EdgeArena.
#[component]
pub fn PlogDiagram() -> Element {
    // DOT graph definition describing the provenance log structure and relationships.
    let dot_graph = r#"
        digraph ProvenanceLog {
            rankdir="TB";
            label="Provenance Log and VLAD";

            subgraph cluster_dht {
                label="Distributed Hash Table";
                rankdir="LR";
                vlad [label="VLAD", nodetype="Vlad"];
                mutable_value [label="Mutable Value
<CID>", nodetype="MutableValue"];
            }

            subgraph cluster_cas {
                label="Content Addressable Storage";
                rankdir="LR";
                wasm [label="WASM
(module...)", nodetype="Wasm"];
                subgraph cluster_entries {
                    label="Entries";
                    rankdir="TB";
                    foot [label="Foot
Seqno 0", nodetype="Foot"];
                    head [label="Head
Seqno 1", nodetype="Head"];
                }
            }

            // Edge definitions based on the diagram's arrows and labels.
            vlad -> mutable_value [label="maps to"];
            vlad -> wasm [label="<Sig of>
<WASM CID>"];
            mutable_value -> head [label="references"];
            wasm -> foot [label="verifies"];
            wasm -> head [label="verifies"];
            head -> foot [label="Prev"];
        }
    "#;

    let graph_data = parse_graph(dot_graph).unwrap_or_default();

    rsx! {
        div {
            class: "mt-6 border border-gray-300 rounded-xl p-4 font-sans",
            h2 { class: "text-xl font-bold mb-4 text-center", "{graph_data.label.clone().unwrap_or_default()}" }

            EdgeArena {
                edges: graph_data.edges.clone(),
                Graph {
                    graph: graph_data
                }
            }
        }
    }
}
