//! Generic approach where any component can become a DOT node renderer by implementing a trait
use crate::{
    components::edge_renderer::{self, EdgeArena},
    graph_data::{GraphData, NodeData},
};
use dioxus::prelude::*;

/// A trait for components that can render DOT graph nodes
pub trait DotNodeRenderer {
    /// Render a specific node based on its data
    fn render_node(&self, node: &NodeData) -> Element;
}

/// Props for the DotGraph component
#[derive(Clone, Props, PartialEq)]
pub struct DotGraphProps<R: DotNodeRenderer + PartialEq + 'static> {
    /// The DOT content to render
    pub dot: String,

    /// Custom renderer for nodes
    pub renderer: R,

    /// Optional class for the container
    #[props(!optional)]
    pub class: Option<String>,
}

/// Component to render a DOT graph with custom node rendering
#[component]
pub fn DotGraph<R: DotNodeRenderer + PartialEq + 'static>(props: DotGraphProps<R>) -> Element {
    // Parse the DOT string
    let graph_result = dot_parser::ast::Graph::<(&str, &str)>::try_from(props.dot.as_str());

    // Handle parsing errors
    if let Err(err) = &graph_result {
        return rsx! {
            div {
                class: "p-4 bg-red-100 text-red-700 rounded",
                "Error parsing DOT: {err}"
            }
        };
    }

    // Convert to our graph data format
    let graph = GraphData::from_ast(&graph_result.unwrap());

    rsx! {
        div {
            class: "relative {props.class.clone().unwrap_or_default()}",
            id: "graph-container",
            style: "position: relative;",

            // Graph title if available
            if let Some(label) = &graph.label {
                h2 {
                    class: "text-lg font-bold mb-4 text-center",
                    "{label}"
                }
            }

            EdgeArena {
                edges: graph.edges.clone(),
                children: render_graph_content(&graph, &props.renderer)
            }
        }
    }
}

/// Helper function to recursively render graph content WITHOUT edges
fn render_graph_content<R: DotNodeRenderer + PartialEq>(
    graph: &GraphData,
    renderer: &R,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-6",

            // Render subgraphs recursively
            {graph.subgraphs.iter().map(|subgraph| {
                let style_class = match subgraph.style.as_deref() {
                    Some("dashed") => "border-dashed",
                    Some("dotted") => "border-dotted",
                    _ => "border-solid",
                };

                rsx! {
                    div {
                        id: "{subgraph.id}",
                        class: "relative p-4 m-2 bg-slate-50 border-2 {style_class} border-slate-300 rounded-lg",
                        "data-subgraph": "true",

                        // Subgraph label
                        if let Some(label) = &subgraph.label {
                            div {
                                class: "absolute -top-3 left-4 px-2 bg-slate-50 text-sm font-bold",
                                "{label}"
                            }
                        }

                        // Recursively render the subgraph's content
                        {render_graph_content(subgraph, renderer)}
                    }
                }
            })}

            // Render nodes in this graph level
            if !graph.nodes.is_empty() {
                div {
                    class: "flex flex-row flex-wrap items-center gap-4 justify-start",
                    {graph.nodes.iter().map(|node| {
                        rsx! {
                            div {
                                id: "{node.id}",
                                "data-node": "true",
                                {renderer.render_node(node)}
                            }
                        }
                    })}
                }
            }

            // NO edge rendering here - all edges are rendered at the top level
        }
    }
}
