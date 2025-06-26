use super::edge_renderer::AllEdgesWithMounted;
use crate::graph_data::GraphData;
use dioxus::prelude::*;
use std::collections::HashSet;

#[component]
pub fn Canvas(graph_data: GraphData) -> Element {
    rsx! {
        div {
            class: "relative w-full h-full overflow-auto p-8 flex flex-col items-center justify-center",
            "data-canvas": "true",
            div {
                class: "bg-white rounded-xl shadow-lg p-6 min-w-[500px] flex flex-wrap items-start justify-center",
                GraphLabelComponent { graph_data: graph_data.clone() }
                GraphContentComponent { graph_data: graph_data.clone() }
                AllEdgesWithMounted { edges: graph_data.edges.clone() }
            }
        }
    }
}

#[component]
fn GraphLabelComponent(graph_data: GraphData) -> Element {
    let graph_label = graph_data
        .label
        .clone()
        .unwrap_or_else(|| "Graph".to_string());

    rsx! {
        h2 {
            class: "w-full text-center text-xl font-bold text-slate-700 mb-4 border-b pb-2",
            "{graph_label}"
        }
    }
}

#[component]
fn GraphContentComponent(graph_data: GraphData) -> Element {
    let data = graph_data;

    // Get standalone nodes (not in any subgraph)
    let nodes_in_subgraphs: HashSet<_> = data
        .subgraphs
        .iter()
        .flat_map(|sg| sg.nodes.iter().cloned())
        .collect();

    let standalone_nodes: Vec<_> = data
        .nodes
        .iter()
        .filter(|node| !nodes_in_subgraphs.contains(&node.id))
        .collect();

    rsx! {
        // Render subgraphs
        {data.subgraphs.iter().map(|subgraph| {
            let subgraph_id = &subgraph.id;
            let label = subgraph.label.as_deref().unwrap_or("");
            let border_style = if subgraph.style.as_deref() == Some("dashed") { "border-dashed" } else { "border-solid" };

            // Find nodes for this subgraph
            let subgraph_nodes: Vec<_> = data.nodes.iter()
                .filter(|node| subgraph.nodes.contains(&node.id))
                .collect();

            rsx! {
                div {
                    id: "{subgraph_id}",
                    class: "relative flex flex-wrap rounded-lg p-4 m-3 bg-slate-50 border-2 {border_style} border-slate-300",
                    "data-subgraph": "true",

                    // Subgraph label
                    div {
                        class: "absolute -top-3 left-4 px-2 bg-slate-50 text-sm font-medium text-slate-700",
                        "{label}"
                    }

                    // Render nodes in this subgraph
                    {subgraph_nodes.iter().map(|node| {
                        let node_id = &node.id;
                        let node_label = node.label.as_deref().unwrap_or(&node.id);

                        rsx! {
                            div {
                                id: "{node_id}",
                                class: "bg-white border border-gray-300 rounded-lg p-3 shadow-md hover:shadow-lg transition-all duration-200 m-2 min-w-[120px] cursor-pointer hover:bg-blue-50",
                                "data-node": "true",
                                "{node_id}) {node_label}"
                            }
                        }
                    })}
                }
            }
        })}

        // Render standalone nodes
        {standalone_nodes.iter().map(|node| {
            let node_id = &node.id;
            let node_label = node.label.as_deref().unwrap_or(&node.id);

            rsx! {
                div {
                    id: "{node_id}",
                    class: "bg-white border border-gray-300 rounded-lg p-3 shadow-md hover:shadow-lg transition-all duration-200 m-2 min-w-[120px] cursor-pointer hover:bg-blue-50",
                    "data-node": "true",
                    "{node_id}) {node_label}"
                }
            }
        })}
    }
}
