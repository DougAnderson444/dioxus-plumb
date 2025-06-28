use dioxus::{logger::tracing, prelude::*};
use dioxus_sdk::storage::use_persistent;

mod examples;
use examples::edge_arena::EdgeArenaExample;
use examples::plog::PlogDiagram;
use examples::workflow::WorkflowExample;

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

/// Define the possible examples to display
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum Example {
    ProjectWorkflow,
    EdgeArenaDemo,
    Plog,
}

impl Example {
    // Associated constants for string representations
    const PROJECT_WORKFLOW_STR: &'static str = "Project Workflow (DOT)";
    const EDGE_ARENA_DEMO_STR: &'static str = "Edge Arena Demo";
    const PROVENANCE_LOG_STR: &'static str = "Provenance Log";

    fn to_string(&self) -> &'static str {
        match self {
            Example::ProjectWorkflow => Self::PROJECT_WORKFLOW_STR,
            Example::EdgeArenaDemo => Self::EDGE_ARENA_DEMO_STR,
            Example::Plog => Self::PROVENANCE_LOG_STR,
        }
    }

    // Helper to convert string back to Example enum
    fn from_string(s: &str) -> Option<Self> {
        match s {
            Self::PROJECT_WORKFLOW_STR => Some(Example::ProjectWorkflow),
            Self::EDGE_ARENA_DEMO_STR => Some(Example::EdgeArenaDemo),
            Self::PROVENANCE_LOG_STR => Some(Example::Plog),
            _ => None,
        }
    }

    // Helper to list all variants for dropdown options (manual list)
    fn all_variants() -> Vec<Example> {
        vec![
            Example::ProjectWorkflow,
            Example::EdgeArenaDemo,
            Example::Plog,
        ]
    }
}

/// Component to view different graph examples
#[component]
fn MyGraphViewer() -> Element {
    // Use the use_local_storage hook instead of use_state
    // It takes the storage key and a default value factory
    let mut selected_example =
        use_persistent::<Example>("selected_graph_example", || Example::ProjectWorkflow);

    rsx! {
        div {
            class: "p-4",
            h1 { class: "text-2xl font-bold mb-4", "Graph Examples" }

            div {
                class: "mb-4",
                label {
                    class: "block text-gray-700 text-sm font-bold mb-2",
                    r#for: "example-select",
                    "Select Example:"
                }
                select {
                    id: "example-select",
                    class: "shadow appearance-none border rounded w-full p-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline",
                    value: selected_example.read().to_string(),
                    onchange: move |event| {
                         let value = event.value();

                        tracing::info!("Selected: {:?}", value);

                         if let Some(example) = Example::from_string(&value) {
                             tracing::info!("Setting example to {:?}", example);
                             selected_example.set(example);
                         } else {
                            tracing::warn!("Unknown example selected: {}", value);
                         }
                    },
                    { Example::all_variants().into_iter().map(|ex| {
                        let s = ex.to_string();
                        rsx! {
                            option { value: s, "{s}" }
                        }
                    }) }

                }
            }

            match *selected_example.read() {
                Example::ProjectWorkflow => rsx! { WorkflowExample {} },
                Example::EdgeArenaDemo => rsx! { EdgeArenaExample {} },
                Example::Plog => rsx! { PlogDiagram {} }
            }
        }
    }
}
