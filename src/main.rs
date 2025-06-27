use dioxus_plumb::{
    dot_renderer, edge_renderer::EdgeArena, graph_data::parse_edges, node_renderer,
};

use dioxus::{logger::tracing, prelude::*};
use dot_renderer::DotGraph;
use node_renderer::InteractiveNodeRenderer;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    rsx! {
        // Add stylesheets
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "icon", href: FAVICON }
        MyGraphViewer {}
    }
}

/// Example usage in a component
#[component]
fn MyGraphViewer() -> Element {
    let dot_content = r#"
        digraph G {
            label="Project Workflow";
            
            subgraph cluster_0 {
                label="Planning & Development";
                style="dashed";
                
                subgraph cluster_0_1 {
                    label="Initial Planning";
                    style="dotted";
                    start [label="Start Project"];
                    requirements [label="Requirements"];
                    planning [label="Planning Phase"];
                }
                
                subgraph cluster_0_2 {
                    label="Implementation";
                    style="dotted";
                    design [label="Design"];
                    development [label="Development"];
                }
            }
            
            subgraph cluster_1 {
                label="Quality & Delivery";
                style="dashed";
                
                subgraph cluster_1_1 {
                    label="Quality Assurance";
                    style="dotted";
                    testing [label="Testing"];
                    qa_review [label="QA Review"];
                }
                
                subgraph cluster_1_2 {
                    label="Release";
                    style="dotted";
                    deployment [label="Deployment"];
                    end [label="Project Complete"];
                }
            }
            
            // Connections between phases
            start -> requirements;
            requirements -> planning;
            planning -> design;
            design -> development;
            development -> testing;
            testing -> qa_review;
            qa_review -> deployment;
            deployment -> end;
            
            // Feedback loops
            qa_review -> development [label="Failed QA"];
            testing -> development [label="Failed Tests"];
        }
    "#;

    // Create an interactive renderer with click handler
    let renderer = InteractiveNodeRenderer {
        on_node_click: Some(EventHandler::new(|node_id| {
            println!("Node clicked: {}", node_id);
            // Handle node click, maybe open a detail panel
        })),
    };

    let node_a = "NodeA";
    let node_b = "NodeB";
    let nodes = [node_a, node_b];

    // edges can be derived from the DOT graph or defined manually
    let dot_edges = format!(
        r#"
        digraph G {{
            label="Edge Arena Edges";
            "{node_a}" -> "{node_b}" [label="Edge from {node_a} to {node_b}"];
        }}
        "#,
        node_a = node_a,
        node_b = node_b
    );

    let edges = parse_edges(&dot_edges).unwrap_or_default();

    tracing::debug!("Parsed edges: {:?}", edges);

    rsx! {
        div {
            class: "p-4",
            h1 { class: "text-2xl font-bold mb-4", "Project Workflow Viewer" }

            // Use the DOT renderer with custom styling for subgraphs
            div {
                class: "flex flex-col gap-6",

                DotGraph {
                    dot: dot_content.to_string(),
                    renderer: renderer,
                    class: Some("bg-white rounded-xl shadow-lg".to_string())
                }
            }


            // Example using Dioxus nodes instead of DOT nodes
            // and connecting them with edges
            div {
                class: "mt-6 border border-gray-300 rounded-xl p-4",
                h2 { class: "text-xl font-bold mb-4", "Edge Arena Demo" }

                // EdgeArena is the container for the nodes and edges
                EdgeArena {
                    edges,
                    div {
                        class: "flex flex-col gap-12",
                        "Describe the edges using DOT, but render nodes using Dioxus components",
                        // the DOT edge String we are using: (pre and code )
                        pre { class: "bg-gray-100 rounded p-4",
                            code { class: "whitespace-pre-wrap overflow-wrap-anywhere text-sm font-mono", "{dot_edges}" }
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
}

/// Basic component as "Source A"
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
