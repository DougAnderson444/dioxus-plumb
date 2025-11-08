//! Generic approach where any component can become a DOT node renderer by implementing a trait
use crate::{
    edge_renderer::EdgeArena,
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
pub struct DotGraphProps<R: DotNodeRenderer + Clone + PartialEq + 'static> {
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
pub fn DotGraph<R: DotNodeRenderer + Clone + PartialEq + 'static>(
    props: DotGraphProps<R>,
) -> Element {
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

            // Graph title if available
            if let Some(label) = &graph.label {
                h2 {
                    class: "text-lg font-bold mb-4 text-center",
                    "{label}"
                }
            }

            EdgeArena {
                edges: graph.edges.clone(),
                node_ids: graph.nodes.iter().map(|n| n.id.clone()).collect(),
                GraphContent {
                    graph: graph,
                    renderer: props.renderer.clone(),
                    collapsed: Some(false)
                }
            }
        }
    }
}

#[derive(Clone, Props, PartialEq)]
struct GraphContentProps<R: DotNodeRenderer + Clone + PartialEq + 'static> {
    graph: GraphData,
    renderer: R,
    collapsed: Option<bool>,
}

/// Helper component to recursively render graph content
#[component]
fn GraphContent<R: DotNodeRenderer + Clone + PartialEq + 'static>(
    props: GraphContentProps<R>,
) -> Element {
    let mut is_collapsed = use_signal(|| props.collapsed.unwrap_or(true));

    // Calculate the nesting level to alternate flex direction
    // Count the number of hyphens to determine nesting level
    // e.g., "cluster_1-cluster_2-n3" has nesting level 2
    let nesting_level = &props.graph.id.matches('-').count();

    // Alternate flex-row and flex-col based on nesting level
    let flex_direction = if nesting_level % 2 == 0 {
        "flex-row" // Even levels: horizontal layout
    } else {
        "flex-col" // Odd levels: vertical layout
    };

    let toggle_collapse = move |_| {
        is_collapsed.toggle();
    };

    let style_class = match props.graph.style.as_deref() {
        Some("dashed") => "border-dashed",
        Some("dotted") => "border-dotted",
        _ => "border-solid",
    };

    let container_class = if props.graph.id.starts_with("cluster_") {
        let base_class =
            "relative p-4 m-2 bg-slate-50 border-2 {style_class} border-slate-300 rounded-lg";
        if is_collapsed() {
            format!("{base_class} h-fit w-fit")
        } else {
            base_class.to_string()
        }
    } else {
        "".to_string()
    };

    // concat label with collapsed icon
    let label = format!(
        "{}{}",
        props.graph.label.as_deref().unwrap_or(""),
        if is_collapsed() { " [+] " } else { " [-] " }
    );

    // Main container for the graph or subgraph
    rsx! {
        div {
            id: "{props.graph.id}",
            class: "{container_class}",
            "data-subgraph": if props.graph.id.starts_with("cluster_") { "true" } else { "false" },

            // Clickable label for collapsing/expanding subgraphs
            if props.graph.label.is_some() {
                if props.graph.id.starts_with("cluster_") {
                    div {
                        class: "absolute -top-3 left-4 px-2 bg-slate-50 text-sm font-bold cursor-pointer select-none whitespace-nowrap z-10",
                        onclick: toggle_collapse,
                        "{label}",
                    }
                }
            }

            // Conditionally render children
            if !is_collapsed() {
                div {
                    // Use flexbox with wrapping and alternating direction
                    class: "flex {flex_direction} flex-wrap gap-2 pt-4 w-fit justify-start items-start",

                    // Render subgraphs recursively
                    {props.graph.subgraphs.iter().map(|subgraph| {
                        rsx! {
                            GraphContent {
                                graph: subgraph.clone(),
                                renderer: props.renderer.clone(),
                                collapsed: None
                            }
                        }
                    })}

                    // Render nodes in this graph level with w-fit
                    {props.graph.nodes.iter().map(|node| {
                        rsx! {
                            div {
                                id: "{node.id}",
                                "data-node": "true",
                                // Use w-fit to minimize width but ensure minimum readability
                                class: "w-fit h-fit",
                                {props.renderer.render_node(node)}
                            }
                        }
                    })}
                }
            }
        }
    }
}
