mod perfect_arrows;

use dioxus::logger::tracing;
use dioxus::prelude::*;
use dot_parser::{
    ast,
    canonical::{self},
};
use perfect_arrows::{get_box_to_box_arrow, ArrowOptions, Pos2, Vec2};
use std::{collections::HashMap, f64::consts::PI};
use wasm_bindgen::{prelude::*, JsCast};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

type Att = (&'static str, &'static str);

/// Newtype for the EdgeStmt since they
#[derive(Clone, Debug, PartialEq)]
struct Edge {
    id: String,
    source: String,
    target: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Rect {
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
}

/// Struct for subgraphs
#[derive(Clone, Debug, PartialEq)]
struct SubgraphData {
    id: String,
    label: String,
    style: String,
}
fn main() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    let initial_dot = r#"digraph G {
        subgraph cluster_0 {
            label="Process A";
            style="dashed";
            1 [label="Start"];
            2 [label="Process"];
            3 [label="Decision"];
            1 -> 2;
            2 -> 3;
        }
        
        subgraph cluster_1 {
            label="Process B";
            style="dashed";
            4 [label="Output"];
            5 [label="End"];
            4 -> 5;
        }
        
        3 -> 4;
        3 -> 5 [label="bypass"];
    }"#;

    // let ast = ast::Graph::try_from(graph_str).unwrap();
    // let graph = canonical::Graph::from(ast.clone());
    //
    // for edge in graph.edges.set {
    //     println!("{} -> {}", edge.from, edge.to);
    // }
    //
    // let handle = |a_list: &ast::AttrList<(&str, &str)>| {
    //     for element in a_list.elems.iter() {
    //         for elem in &element.elems {
    //             println!("Attribute: {} = {}", elem.0, elem.1);
    //         }
    //     }
    // };
    //
    // let stmt_list = &ast.stmts;
    //
    // for stmt in stmt_list {
    //     match stmt {
    //         ast::Stmt::NodeStmt(node_stmt) => {
    //             println!("Node: {}", node_stmt.node.id);
    //         }
    //         ast::Stmt::AttrStmt(ast::AttrStmt::Graph(attr_list)) => {
    //             handle(attr_list);
    //         }
    //         ast::Stmt::AttrStmt(ast::AttrStmt::Node(attr_list)) => {
    //             handle(attr_list);
    //         }
    //         ast::Stmt::Subgraph(subgraph) => {
    //             println!(
    //                 "Subgraph: {}",
    //                 subgraph.id.clone().unwrap_or("Unidentified".to_string())
    //             );
    //             // recursively handle StmtList in subgraph
    //             // &subgraph.stmts
    //         }
    //         _ => {}
    //     }
    // }

    let ast = use_signal(|| {
        ast::Graph::try_from(initial_dot).unwrap_or_else(|_| {
            // Fallback to empty graph on parse error
            ast::Graph::try_from("digraph G { }").unwrap()
        })
    });

    // Main render
    rsx! {
        // Add stylesheets
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "icon", href: FAVICON }

        // Main container
        div {
            class: "relative w-screen h-screen bg-gradient-to-br from-slate-50 to-slate-200 overflow-hidden flex flex-col",

            // Header
            div {
                class: " bg-white shadow-sm border-b border-slate-200 p-4",
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
                Canvas { ast }
            }
        }
    }
}

/// Canvas component with "data-canvas": "true", data attribute
#[component]
fn Canvas(ast: Signal<dot_parser::ast::Graph<Att>>) -> Element {
    let graph = canonical::Graph::from(ast.read().clone());
    rsx! {
        div {
            class: "relative w-full h-full overflow-auto p-8 flex flex-col items-center justify-center",
            "data-canvas": "true",
            div {
                class: "bg-white rounded-xl shadow-lg p-6 min-w-[500px] flex flex-wrap items-start justify-center",
                StmtListComponent { stmts: ast.read().stmts.clone() }
                AllEdgesWithMounted { edges: graph.edges.set.iter().map(|edge| Edge {
                    id: format!("{}-{}", edge.from, edge.to),
                    source: edge.from.clone(),
                    target: edge.to.clone(),
                }).collect() }
            }
        }
    }
}

// Updated AllNodes component - add data attributes for easier selection
#[component]
fn StmtListComponent(stmts: ast::StmtList<Att>) -> Element {
    rsx! {
        {stmts.into_iter().map(|stmt| {
            match stmt {
                ast::Stmt::NodeStmt(node_stmt) => {
                    let node = &node_stmt.node;
                    // get label from attributes, if option exists
                    let label = node_stmt
                        .attr
                        .map(|attr| {
                            attr.flatten()
                                .into_iter()
                                .find_map(|(key, value)| {
                                    if key == "label" {
                                        // Remove quotes from the label
                                        Some(value.trim_matches('"'))
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or("No label")
                        })
                        .unwrap_or("No label");

                    tracing::debug!("Rendering node: {}", node.id);

                    rsx! {
                        div {
                            id: "{node.id}",
                            class: "bg-white border border-gray-300 rounded-lg p-3 shadow-md hover:shadow-lg transition-all duration-200 m-2 min-w-[120px] cursor-pointer hover:bg-blue-50",
                            "data-node": "true",
                            "{node.id}) {label}"
                        }
                    }
                }
                ast::Stmt::Subgraph(subgraph) => {
                    // Extract subgraph attributes
                    let subgraph_id = subgraph.id.clone().unwrap_or_else(|| "cluster".to_string());

                    // Extract label and style from attributes
                    let mut label = "";
                    let mut style = "solid";

                    // Process all statements to find attributes
                    for sub_stmt in &subgraph.stmts {
                        match sub_stmt {
                            // Handle the IDEq variant which contains attributes like label and style
                            ast::Stmt::IDEq(attr_name, attr_value) => {
                                if attr_name == "label" {
                                    label = attr_value.trim_matches('"');
                                } else if attr_name == "style" {
                                    style = attr_value.trim_matches('"');
                                }
                            }
                            // Keep the original handling for AttrStmt as a fallback
                            ast::Stmt::AttrStmt(ast::AttrStmt::Graph(attr_list)) => {
                                for element in attr_list.elems.iter() {
                                    for elem in &element.elems {
                                        if elem.0 == "label" {
                                            label = elem.1.trim_matches('"');
                                        } else if elem.0 == "style" {
                                            style = elem.1.trim_matches('"');
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    let border_style = if style == "dashed" { "border-dashed" } else { "border-solid" };

                    // Recursively render subgraph stmts in a container
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

                            // Render all children of the subgraph
                            StmtListComponent { stmts: subgraph.stmts.clone() }
                        }
                    }
                }
                _ => {
                    // Handle other statements if needed
                    rsx! {}
                }
            }
        })}
    }
}

#[component]
fn AllEdgesWithMounted(edges: Vec<Edge>) -> Element {
    let mut arrow_paths = use_signal(HashMap::<String, EdgeSvgData>::new);
    let edges_ref = use_signal(|| edges.clone());
    let mut initial_load = use_signal(|| true);

    // Store window dimensions to trigger recalculation
    let mut window_size = use_signal(|| {
        let window = web_sys::window().unwrap();
        (
            window.inner_width().unwrap().as_f64().unwrap() as i32,
            window.inner_height().unwrap().as_f64().unwrap() as i32,
        )
    });

    // Function to calculate all arrows
    let calculate_arrows = move || {
        let edges_to_calculate = edges_ref.read().clone();
        spawn(async move {
            // Small delay to ensure all sibling elements are rendered
            if *initial_load.read() {
                gloo_timers::future::TimeoutFuture::new(150).await;
            }
            initial_load.set(false);

            let mut new_paths = HashMap::new();
            for edge in edges_to_calculate.iter() {
                if let Ok(svg_data) = generate_arrow_path_safe(edge) {
                    new_paths.insert(edge.id.clone(), svg_data);
                }
            }
            arrow_paths.set(new_paths);
        });
    };

    // Set up resize listener using use_effect
    use_effect(move || {
        let window = web_sys::window().unwrap();

        // Use Box::new with FnMut and wrap it as a Box<dyn FnMut()>
        let update_size = Closure::wrap(Box::new(move || {
            let window = web_sys::window().unwrap();
            let w = window.inner_width().unwrap().as_f64().unwrap() as i32;
            let h = window.inner_height().unwrap().as_f64().unwrap() as i32;
            window_size.set((w, h));
        }) as Box<dyn FnMut()>); // Note: FnMut instead of Fn

        window
            .add_event_listener_with_callback("resize", update_size.as_ref().unchecked_ref())
            .unwrap();

        // Keep the closure alive for the lifetime of the component
        update_size.forget();

        // Calculate arrows initially
        calculate_arrows();
    });

    // React to window size changes
    use_effect(move || {
        // The dependency on window_size will cause this to run when window size changes
        let _ = *window_size.read();
        calculate_arrows();
    });

    rsx! {
        svg {
            class: "absolute top-0 left-0 w-full h-full pointer-events-none overflow-visible",
            {arrow_paths.read().iter().map(|(edge_id, svg_data)| {
                rsx! {
                    g {
                        key: "{edge_id}",
                        path {
                            d: "{svg_data.path}",
                            fill: "none",
                            stroke: "#4b5563", // gray-600
                            "stroke-width": "2.5"
                        }
                        polygon {
                            points: "-6,-4 0,0 -6,4",
                            fill: "#4b5563", // gray-600
                            transform: "{svg_data.arrow_transform}"
                        }
                    }
                }
            })}
        }
    }
}

#[derive(Clone, Debug)]
struct EdgeSvgData {
    path: String,
    arrow_transform: String,
}

fn generate_arrow_path_safe(edge: &Edge) -> Result<EdgeSvgData, String> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Much simpler and faster - direct ID lookup
    let source_el = document
        .get_element_by_id(&edge.source)
        .ok_or("Source element not found")?;

    let target_el = document
        .get_element_by_id(&edge.target)
        .ok_or("Target element not found")?;

    let canvas_el = source_el
        .closest("[data-canvas]")
        .map_err(|_| "Canvas query failed")?
        .ok_or("Canvas element not found")?;

    // Get coordinates
    let source = get_coords(&source_el);
    let target = get_coords(&target_el);
    let canvas = get_coords(&canvas_el);

    tracing::debug!("Source: {:?}, Target: {:?}", source, target);

    let x_0 = source.left - canvas.left;
    let y_0 = source.top - canvas.top;
    let x_1 = target.left - canvas.left;
    let y_1 = target.top - canvas.top;

    let w_0 = source.right - source.left;
    let h_0 = source.bottom - source.top;
    let w_1 = target.right - target.left;
    let h_1 = target.bottom - target.top;

    let start = Pos2 { x: x_0, y: y_0 }; // Use top-left instead of center
    let end = Pos2 { x: x_1, y: y_1 }; // Use top-left instead of center

    tracing::debug!("Start: {:?}, End: {:?}", start, end);

    let start_size = Vec2 { x: w_0, y: h_0 };

    let end_size = Vec2 { x: w_1, y: h_1 };

    let options = ArrowOptions::default();

    let (
        Pos2 { x: sx, y: sy },
        Pos2 { x: cx, y: cy },
        Pos2 { x: ex, y: ey },
        angle_end,
        _angle_start,
        _angle_center,
    ) = get_box_to_box_arrow(start, start_size, end, end_size, options);

    let path = format!(
        "M{sx},{sy} Q{cx},{cy} {ex},{ey}",
        sx = sx,
        sy = sy,
        cx = cx,
        cy = cy,
        ex = ex,
        ey = ey
    );

    let end_angle_as_degrees = angle_end * (180.0 / PI);
    let arrow_transform = format!("translate({}, {}) rotate({})", ex, ey, end_angle_as_degrees);

    Ok(EdgeSvgData {
        path,
        arrow_transform,
    })
}

fn get_coords(el: &web_sys::Element) -> Rect {
    let rect = el.get_bounding_client_rect();
    Rect {
        top: rect.top() + web_sys::window().unwrap().page_y_offset().unwrap_or(0.0),
        right: rect.right() + web_sys::window().unwrap().page_x_offset().unwrap_or(0.0),
        bottom: rect.bottom() + web_sys::window().unwrap().page_y_offset().unwrap_or(0.0),
        left: rect.left() + web_sys::window().unwrap().page_x_offset().unwrap_or(0.0),
    }
}
