//! Graph Visualization Component
use crate::{components::canvas::Canvas, graph_data::GraphData};
use dioxus::prelude::*;

#[component]
pub fn Graph(graph_data: GraphData) -> Element {
    // Main render
    rsx! {

        // Main container
        div {
            class: "relative w-screen h-screen bg-gradient-to-br from-slate-50 to-slate-200 overflow-hidden flex flex-col",

            // Header
            div {
                class: "bg-white shadow-sm border-b border-slate-200 p-4",
                div {
                    class: "container mx-auto flex items-center justify-between",
                    div {
                        h1 {
                            class: "text-xl font-bold text-slate-800",
                            "Graph Visualization"
                        }
                        p {
                            class: "text-sm text-slate-600",
                            "Interactive nodes with perfect arrows and nested subgraphs"
                        }
                    }
                }
            }

            // Canvas component
            div {
                class: "flex-1 overflow-hidden",
                Canvas { graph_data }
            }
        }
    }
}
