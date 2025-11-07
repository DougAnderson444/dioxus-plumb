//! A simple REPL for rendering DOT graphs with Dioxus and Dioxus-Plumb.
//! Edit the existing DOT code or upload a .dot file to visualize different graphs.
use dioxus::{logger::tracing, prelude::*};
use dioxus_plumb::{
    dot_renderer::DotGraph,
    edge_renderer::EdgeArena,
    edge_renderer::EdgeData,
    graph_data::{parse_graph, GraphData},
    node_renderer::InteractiveNodeRenderer,
};
use std::collections::HashSet;

#[component]
pub fn DotRepl() -> Element {
    // State for the DOT input
    let mut dot_input = use_signal(|| {
        String::from(
            r#"digraph G {
    A [label="Box A"];
    A -> B;
    B -> C;
    C -> A;
}"#,
        )
    });

    // State for error messages and parsed data
    let mut error = use_signal(|| Option::<String>::None);
    let mut graph_data = use_signal(|| Option::<GraphData>::None);
    let mut edges = use_signal(|| Vec::<EdgeData>::new());
    let mut node_ids = use_signal(|| HashSet::<String>::new());

    // Function to parse DOT and extract nodes/edges
    let mut parse_dot = move || {
        match parse_graph(&dot_input.read()) {
            Ok(graph) => {
                // Extract all node IDs from the graph
                let mut nodes = HashSet::new();
                extract_node_ids(&graph, &mut nodes);

                // Get the edges
                let extracted_edges = graph.edges.clone();

                // Update the state
                node_ids.set(nodes);
                edges.set(extracted_edges);
                graph_data.set(Some(graph));
                error.set(None);
            }
            Err(err) => {
                error.set(Some(err));
                graph_data.set(None);
                edges.set(Vec::new());
                node_ids.set(HashSet::new());
            }
        }
    };

    // Helper function to extract node IDs from the graph recursively
    fn extract_node_ids(graph: &GraphData, nodes: &mut HashSet<String>) {
        // Add nodes from this level
        for node in &graph.nodes {
            nodes.insert(node.id.clone());
        }

        // Add edge nodes (may not be explicitly defined as nodes)
        for edge in &graph.edges {
            nodes.insert(edge.source.clone());
            nodes.insert(edge.target.clone());
        }

        // Recursively process subgraphs
        for subgraph in &graph.subgraphs {
            extract_node_ids(subgraph, nodes);
        }
    }

    // Parse the initial DOT input
    use_effect(move || {
        parse_dot();
    });

    // Function to handle rendering button click
    let handle_render = move |_| {
        // Basic validation for DOT syntax
        if !dot_input.read().contains("{") || !dot_input.read().contains("}") {
            error.set(Some("Invalid DOT: Missing curly braces".to_string()));
            return;
        }

        match parse_graph(&dot_input.read()) {
            Ok(graph) => {
                // Check for potential issues with complex graphs
                if graph.edges.len() > 50 {
                    // Just a warning, still allow rendering
                    error.set(Some(format!(
                        "Complex graph with {} edges - rendering may be slow",
                        graph.edges.len()
                    )));
                } else {
                    error.set(None);
                }

                // Count duplicate node IDs (nodes appearing in multiple places)
                let mut node_ids = HashSet::new();
                let mut duplicates = 0;

                for edge in &graph.edges {
                    // Check source and target nodes
                    if !node_ids.insert(edge.source.clone()) {
                        duplicates += 1;
                    }
                    if !node_ids.insert(edge.target.clone()) {
                        duplicates += 1;
                    }
                }

                if duplicates > 0 {
                    tracing::info!("Graph contains {} duplicate node references", duplicates);
                }
            }
            Err(err) => {
                error.set(Some(format!("Error parsing DOT: {}", err)));
            }
        }
    };

    // File upload handler
    let handle_file_upload = move |evt: Event<FormData>| {
        tracing::info!("File upload event: {:?}", evt);
        if let Some(file_engine) = evt.files() {
            tracing::info!("Files uploaded: {:?}", file_engine.files());
            let engine_clone = file_engine.clone();
            // Read the file content asynchronously
            spawn(async move {
                if let Some(file) = file_engine.files().first() {
                    tracing::info!("Reading file: {}", file);
                    engine_clone
                        .read_file_to_string(file)
                        .await
                        .map(|v| {
                            tracing::info!("File read successfully: {}", file);
                            dot_input.set(v);
                        })
                        .unwrap_or_else(|| {
                            tracing::error!("Failed to read file: {}", file);
                            error.set(Some("Failed to read file".to_string()));
                        });
                }
            });
        }
    };
    // Create an interactive renderer
    let interactive_renderer = InteractiveNodeRenderer {
        on_node_click: Some(EventHandler::new(|node_id| {
            tracing::info!("Node clicked: {node_id}");
        })),
    };

    rsx! {
        div {
            class: "flex flex-col gap-6",
            h2 { class: "text-xl font-bold mb-4", "DOT REPL" }

            p {
                class: "text-sm text-gray-600",
                "Enter a DOT graph definition below or upload a DOT file. The renderer will visualize your graph and let you interact with it."
            }

            div {
                class: "flex flex-col md:flex-row gap-4",

                // Input area
                div {
                    class: "flex-1",
                    div {
                        class: "mb-4",
                        label {
                            class: "block text-gray-700 text-sm font-bold mb-2",
                            "DOT Input:"
                        }
                        textarea {
                            class: "shadow appearance-none border rounded w-full h-64 p-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline font-mono",
                            value: "{dot_input}",
                            oninput: move |evt| dot_input.set(evt.value().clone()),
                        }
                    }

                    // File upload
                    div {
                        class: "mb-4",
                        label {
                            class: "block text-gray-700 text-sm font-bold mb-2",
                            "Upload DOT File:"
                        }
                        input {
                            r#type: "file",
                            accept: ".dot,.gv,text/plain",
                            onchange: handle_file_upload,
                            class: "block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded file:border-0 file:text-sm file:font-semibold file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100"
                        }
                    }

                    // Render button
                    button {
                        class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline",
                        onclick: handle_render,
                        "Render Graph"
                    }

                    // Error display
                    if let Some(err_msg) = error.read().as_ref() {
                        div {
                            class: "mt-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded",
                            "{err_msg}"
                        }
                    }
                }

                // Output area with graph visualization
                div {
                    class: "flex-1 border rounded-xl shadow-lg bg-white",
                    if error.read().is_none() {
                        DotGraph {
                            dot: dot_input.read().clone(),
                            renderer: interactive_renderer.clone(),
                            class: Some("w-full min-h-[400px]".to_string()),
                            // on_error: Some(EventHandler::new(move |err: String| {
                            //     error.set(Some(format!("DOT Rendering Error: {}", err)));
                            // })),
                        }
                    } else {
                        div {
                            class: "flex items-center justify-center h-full min-h-[400px] text-gray-500",
                            "Fix the DOT syntax to render graph"
                        }
                    }
                }
            }
        }
    }
}
