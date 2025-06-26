mod components;
mod connectable;
mod graph_data;
mod perfect_arrows;

mod dot_renderer;
mod node_renderer;

use dioxus::prelude::*;
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
                start [label="Start Project"];
                planning [label="Planning Phase"];
                development [label="Development"];
            }
            
            subgraph cluster_1 {
                label="Quality & Delivery";
                style="dashed";
                testing [label="Testing"];
                deployment [label="Deployment"];
                end [label="Project Complete"];
            }
            
            start -> planning [label="begin"];
            planning -> development;
            development -> testing;
            testing -> deployment;
            testing -> development [label="Failed Tests"];
            deployment -> end;
        }
    "#;

    // Create an interactive renderer with click handler
    let renderer = InteractiveNodeRenderer {
        on_node_click: Some(EventHandler::new(|node_id| {
            println!("Node clicked: {}", node_id);
            // Handle node click, maybe open a detail panel
        })),
    };

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
        }
    }
}
