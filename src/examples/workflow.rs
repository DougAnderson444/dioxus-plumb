use dioxus::prelude::*;
use dioxus_plumb::{dot_renderer::DotGraph, node_renderer::InteractiveNodeRenderer};

#[component]
pub fn WorkflowExample() -> Element {
    // DIAGRAM EXAMPLE 1: Project Workflow (DOT)
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

    // Create an interactive renderer with click handler for DotGraph
    let dot_renderer_config = InteractiveNodeRenderer {
        on_node_click: Some(EventHandler::new(|node_id| {
            println!("Node clicked: {}", node_id);
            // Handle node click, maybe open a detail panel
        })),
    };

    rsx! {
        div {
            class: "flex flex-col gap-6",

            h2 { class: "text-xl font-bold mb-4", "Project Workflow Viewer" }

            DotGraph {
                dot: dot_content.to_string(),
                renderer: dot_renderer_config,
                class: Some("bg-white rounded-xl shadow-lg".to_string())
            }
        }
    }
}
