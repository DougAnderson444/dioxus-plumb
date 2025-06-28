use dioxus::prelude::*;
use dioxus_plumb::{edge_renderer::EdgeArena, graph_data::parse_edges};

// Move the Basic component here as it's specific to this example
#[component]
fn Basic(id: String, children: Element) -> Element {
    rsx! {
        div {
            id,
            class: "p-4 bg-slate-300/60 rounded-lg",
            h2 { class: "text-xl font-semibold", "Source A" }
            p { "This is the content of Source {id}." }
            {children}
        }
    }
}

#[component]
pub fn EdgeArenaExample() -> Element {
    // DIAGRAM EXAMPLE 2: Edge Arena Demo
    let node_a = "NodeA";
    let node_b = "NodeB";
    let nodes = [node_a, node_b];

    // edges can be derived from the DOT graph or defined manually
    let edge_arena_dot_edges = format!(
        r#"
        digraph G {{
            label="Edge Arena Edges";
            "{node_a}" -> "{node_b}" [label="Edge from {node_a} to {node_b}"];
        }}
        "#,
        node_a = node_a,
        node_b = node_b
    );

    let edge_arena_edges = parse_edges(&edge_arena_dot_edges).unwrap_or_default();

    rsx! {
        div {
             class: "mt-6 border border-gray-300 rounded-xl p-4",
             h2 { class: "text-xl font-bold mb-4", "Edge Arena Demo" }

             // EdgeArena is the container for the nodes and edges
             EdgeArena {
                 edges: edge_arena_edges,
                 div {
                     class: "flex flex-col gap-12",
                     "Describe the edges using DOT, but render nodes using Dioxus components",
                     // the DOT edge String we are using: (pre and code )
                     pre { class: "bg-gray-100 rounded p-4",
                         code { class: "whitespace-pre-wrap overflow-wrap-anywhere text-sm font-mono", "{edge_arena_dot_edges}" }
                     }

                     // Simple way to render nodes using Dioxus components
                     {nodes.iter().map(|&node| {
                         rsx! {
                             Basic {
                                 id: node.to_string(),
                                 h2 { class: "text-xl", "Child Node {node}" }
                             }
                         }
                     })}
                 }
             }
         }
    }
}
