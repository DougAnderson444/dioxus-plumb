use crate::dot_renderer::DotNodeRenderer;
use crate::graph_data::NodeData;
use dioxus::prelude::*;

// A simple default renderer for DOT nodes
pub struct DefaultNodeRenderer;

impl DotNodeRenderer for DefaultNodeRenderer {
    fn render_node(&self, node: &NodeData) -> Element {
        let label = node.label.as_deref().unwrap_or(&node.id);

        rsx! {
            div {
                class: "bg-white border border-gray-300 rounded p-3 m-2 shadow",
                "{label}"
            }
        }
    }
}

// A fancy renderer with different styles based on node attributes
pub struct FancyNodeRenderer;

impl DotNodeRenderer for FancyNodeRenderer {
    fn render_node(&self, node: &NodeData) -> Element {
        let label = node.label.as_deref().unwrap_or(&node.id);

        // Different styles based on node ID or label
        let style = if node.id.contains("start") || label.to_lowercase().contains("start") {
            "bg-green-100 border-green-500 text-green-700"
        } else if node.id.contains("end") || label.to_lowercase().contains("end") {
            "bg-red-100 border-red-500 text-red-700"
        } else if node.id.contains("decision") || label.to_lowercase().contains("decision") {
            "bg-yellow-100 border-yellow-500 text-yellow-700"
        } else {
            "bg-blue-100 border-blue-500 text-blue-700"
        };

        rsx! {
            div {
                class: "border-2 rounded-lg p-4 m-2 shadow-md {style}",
                h3 {
                    class: "font-bold mb-1",
                    "Node: {node.id}"
                }
                p {
                    "{label}"
                }
            }
        }
    }
}

// A highly interactive component renderer
#[derive(Clone, PartialEq)]
pub struct InteractiveNodeRenderer {
    // Could include state handlers, event callbacks, etc.
    pub on_node_click: Option<EventHandler<String>>,
}

impl DotNodeRenderer for InteractiveNodeRenderer {
    fn render_node(&self, node: &NodeData) -> Element {
        let node_id = node.id.clone();
        let label = node.label.as_deref().unwrap_or(&node.id);
        let on_click = self.on_node_click;

        // More compact styling that works well with w-fit
        rsx! {
            div {
                class: "bg-white border border-gray-300 rounded-lg p-2 shadow-sm hover:shadow-md transition-all duration-200 cursor-pointer text-center w-fit mx-auto",
                onclick: move |_| {
                    if let Some(handler) = &on_click {
                        handler.call(node_id.clone());
                    }
                },

                div {
                    class: "font-medium text-sm break-words",
                    pre {
                        class: "whitespace-pre-wrap",
                        "{label}"
                    }
                }
            }
        }
    }
}
