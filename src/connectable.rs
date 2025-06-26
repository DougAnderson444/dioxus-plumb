//! Connectable trait and wrapper for Dioxus components
use crate::graph_data::EdgeData;
use dioxus::prelude::*;

/// A trait for Dioxus components that can be connected with edges
pub trait Connectable {
    /// Get the ID of this connectable component
    fn get_id(&self) -> String;

    /// Optional method to get custom connection points
    fn get_connection_points(&self) -> Option<Vec<(f64, f64)>> {
        None
    }

    /// Create edges to other connectables
    fn connect_to(&self, target_id: &str, label: Option<String>) -> EdgeData {
        EdgeData {
            id: format!("{}-{}", self.get_id(), target_id),
            source: self.get_id(),
            target: target_id.to_string(),
            label,
        }
    }
}

/// Wrap a component to make it connectable
#[component]
pub fn ConnectableWrapper(
    id: String,
    #[props(!optional)] class: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div {
            id: "{id}",
            class: class.clone().unwrap_or_default(),
            "data-connectable": "true",
            {children}
        }
    }
}
