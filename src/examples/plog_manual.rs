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
use dioxus_plumb::{edge_renderer::EdgeArena, graph_data::parse_graph};

// Move the Basic component here as it's specific to this example
#[component]
fn Basic(id: String, children: Element) -> Element {
    rsx! {
        div {
            id,
            class: "h-fit p-4 bg-slate-300/60 rounded-lg",
            h2 { class: "font-semibold", "{id}" }
            {children}
        }
    }
}

/// Smaller component for fields in a Nasic node
#[component]
fn Field(id: String, children: Element, code: Option<bool>) -> Element {
    let class = if code.unwrap_or(false) {
        "p-2 text-sm font-mono bg-neutral-700 text-green-400 rounded-md"
    } else {
        "gap-2 p-2 border-2 bg-slate-50 border-slate-300 rounded-lg"
    };
    rsx! {
        div {
            id: id,
            class: "flex flex-col shadow-md {class} ",
            {children}
        }
    }
}

/// Subgrpagh component for rendering a subgraph with a title
#[component]
fn Subgraph(title: String, id: String, children: Element) -> Element {
    rsx! {
        div {
            id: id,
            class: "relative p-4 m-2 bg-slate-50 border-2 border-dotted border-slate-300 rounded-lg",
            h3 { class: "absolute -top-3 left-4 px-2 bg-slate-50 text-sm font-bold", "{title}" },
            {children}
        }
    }
}

#[component]
pub fn PlogManual() -> Element {
    // DIAGRAM EXAMPLE 2: Edge Arena Demo
    // let node_a = "NodeA";
    // let node_b = "NodeB";
    // let nodes = [node_a, node_b];

    // edges can be derived from the DOT graph or defined manually
    // let edge_arena_dot_edges = format!(
    //     r#"
    //     digraph G {{
    //         label="Edge Arena Edges";
    //         "{node_a}" -> "{node_b}" [label="Edge from {node_a} to {node_b}"];
    //     }}
    //     "#,
    //     node_a = node_a,
    //     node_b = node_b
    // );

    // Node ids for Provenance Log Diagram
    let vlad_sig = "VLAD Signature";
    let wasm_cid = "WASM CID";
    let vlad = "VLAD";
    let mutable_value = "Mutable Value";
    let head = "Head";
    let foot = "Foot";
    let first_lock_script = "First Lock Script";
    let first_lock = "First Lock CID";
    let vlad_pubkey = "Vlad Pubkey";
    let head_prev = "Head Prev";
    // seqno 1
    let seqno_1 = "Seqno 1";
    let seqno_0 = "Seqno 0";

    let edge_arena_dot_edges = format!(
        r#"
        digraph G {{
            label="Provenance Log and VLAD";
            node [shape=box, style=filled, fillcolor="\#f0f0f0"];
            edge [fontname="Arial", fontsize=10];

            "{wasm_cid}" -> "{first_lock_script}" [label="Identifies Content of"];
            "{vlad_sig}" -> "{wasm_cid}" [label="of"];
            "{vlad}" -> "{mutable_value}" [label="Maps to"];
            "{first_lock_script}" -> "{foot}" [label="Verifies"];
            "{mutable_value}" -> "{head}" [label="References"];
            "{vlad_pubkey}" -> "{vlad_sig}" [label="Verifies"];
            "{head_prev}" -> "{foot}";
            "{first_lock}" -> "{first_lock_script}" [label="References"];
            "{seqno_0}" -> "{seqno_1}" [label="then"];
        }}
        "#,
    );

    let graph_data = parse_graph(&edge_arena_dot_edges).unwrap_or_default();

    rsx! {
        div {
             class: "mt-6 border border-gray-300 rounded-lg p-4",
             h2 { class: "text-lg font-bold mb-4", "Edge Arena Demo" }

             // EdgeArena is the container for the nodes and edges
             EdgeArena {
                 edges: graph_data.edges,
                 div {
                     class: "flex flex-col gap-12",
                     "Provenance Log and VLAD",

                    // use flexbox to nest the nodes, place a description below each node,
                    // and a title at the center top of each subgraph node
                    div {
                        class: "flex flex-col gap-12",
                        // First subgraph is the DHT, which holds another subgraph (Sig and CId)
                        Subgraph {
                            title: "Distributed Hash Table".to_string(),
                            id: "DHT".to_string(),
                            div {
                                class: "flex flex-row gap-12 items-center justify-between",
                                Subgraph {
                                    title: "VLAD".to_string(),
                                    id: vlad.to_string(),
                                    div {
                                        class: "flex flex-row gap-12",
                                        Field {
                                            id: vlad_sig.to_string(),
                                            span { class: "", "Signature" }
                                        },
                                        Field {
                                            id: wasm_cid.to_string(),
                                            { wasm_cid }
                                        }
                                    }
                                },
                                Field {
                                    id: mutable_value.to_string(),
                                    h2 { class: "text-lg", "Mutable Value" }
                                    span { class: "font-mono", "CID of Head"}
                                }
                            }
                        },
                        // Second subgraph is the Content Addressable Storage
                        Subgraph {
                            title: "Content Addressable Storage".to_string(),
                            id: "CAS".to_string(),
                            div {
                                class: "flex flex-col justify-between gap-12 items-center",
                                Field {
                                    id: first_lock_script.to_string(),
                                    code: Some(true),
                                    "First Lock Script"
                                }
                                // Head and Foot are Entries subgrah
                                Subgraph {
                                    title: "Entries".to_string(),
                                    id: "Entries".to_string(),
                                    div {
                                        class: "flex flex-row justify-between gap-12",
                                        // Foot is subgraph too
                                        Subgraph {
                                            title: "Foot".to_string(),
                                            id: foot.to_string(),
                                            div {
                                                class: "flex flex-col gap-4",
                                                Field {
                                                    id: first_lock.to_string(),
                                                    "First Lock CID",
                                                },
                                                Field {
                                                    id: seqno_0.to_string(),
                                                    "Seqno 0",
                                                },
                                                Field {
                                                    id: "Prev NULL".to_string(),
                                                    "Prev NULL",
                                                },
                                                Field {
                                                    id: vlad_pubkey.to_string(),
                                                    "Vlad Pubkey",
                                                }
                                            }
                                        },
                                        // Head is subgraph too, prev Field points to Foot
                                        Subgraph {
                                            title: "Head".to_string(),
                                            id: head.to_string(),
                                            div {
                                                class: "flex flex-col gap-4",
                                                Field {
                                                    id: "Unlock Script Head".to_string(),
                                                    code: Some(true),
                                                    pre { class: "", "unlock script" }
                                                },
                                                Field {
                                                    id: seqno_1.to_string(),
                                                    "Seqno 1",
                                                },
                                                Field {
                                                    id: head_prev.to_string(),
                                                    "Prev",
                                                }
                                                Field {
                                                    id: "Key Values".to_string(),
                                                    "Key Value Pairs",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                 }
             }
         }
    }
}
