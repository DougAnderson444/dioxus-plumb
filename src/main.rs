mod perfect_arrows;

use dioxus::logger::tracing;
use dioxus::prelude::*;
use dot_parser::{
    ast,
    canonical::{self},
};
use perfect_arrows::{get_box_to_box_arrow, ArrowOptions, Pos2, Vec2};
use std::{collections::HashMap, f64::consts::PI};

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

fn main() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    let initial_dot = r#"digraph G {
        1 [label="Node 1"];
        2 [label="Node 2"];
        1 -> 2;
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
            class: "relative w-screen h-screen bg-slate-100",

            // Title
            div {
                class: "absolute top-4 left-4 text-xl font-bold",
                "Connected Boxes Example"
            }

            // Instructions
            div {
                class: "absolute top-10 left-4 text-sm text-slate-600",
                "Boxes connected with curved perfect arrows"
            }

            // Canvas component
            Canvas { ast }

        }
    }
}

/// Canvas component with "data-canvas": "true", data attribute
#[component]
fn Canvas(ast: Signal<dot_parser::ast::Graph<Att>>) -> Element {
    let graph = canonical::Graph::from(ast.read().clone());
    rsx! {
        div {
            class: "relative h-full inset-0",
            "data-canvas": "true",
            StmtListComponent { stmts: ast.read().stmts.clone() }
            AllEdgesWithMounted { edges: graph.edges.set.iter().map(|edge| Edge {
                id: format!("{}-{}", edge.from, edge.to),
                source: edge.from.clone(),
                target: edge.to.clone(),
            }).collect() }
        }
    }
}

// Updated AllNodes component - add data attributes for easier selection
#[component]
fn StmtListComponent(stmts: ast::StmtList<Att>) -> Element {
    rsx! {
        // Recurively render all node stmts
        // Like in the commented code above, there may be a
        // &subgraph.stmts which would be rendered again by this component
        {stmts.into_iter().map(|stmt| {
            match stmt {
                ast::Stmt::NodeStmt(node_stmt) => {
                    let node = &node_stmt.node;
                    // get label from attributes, if option exists
                    tracing::debug!("Node statement: {:?}", node_stmt);
                    let label = node_stmt
                        .attr
                        .map(|attr| {
                            attr.flatten()
                                .into_iter()
                                .find_map(|(key, value)| {
                                    if key == "label" {
                                        Some(value)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or("No label")
                        })
                        .unwrap_or("No label");

                    tracing::debug!("Rendering node: {}", node.id);
                    let left = node.id.parse::<i32>().unwrap_or(0) * 210;
                    let top = node.id.parse::<i32>().unwrap_or(0) * 190;
                    rsx! {
                        div {
                            id: "{node.id}",
                            class: "absolute bg-white border border-gray-300 rounded p-2 outline-none w-fit",
                            style: format!("left: {}px; top: {}px;", left, top),
                            "{node.id}) {label}"
                        }
                    }
                }
                ast::Stmt::Subgraph(subgraph) => {
                    // Recursively render subgraph stmts
                    rsx! {
                        StmtListComponent { stmts: subgraph.stmts }
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

    // This will be called after the component is mounted to the DOM
    let on_mounted = move |_| {
        let edges_clone = edges.clone();
        spawn(async move {
            // Small delay to ensure all sibling elements are rendered
            gloo_timers::future::TimeoutFuture::new(150).await;

            let mut new_paths = HashMap::new();
            for edge in edges_clone.iter() {
                if let Ok(svg_data) = generate_arrow_path_safe(edge) {
                    new_paths.insert(edge.id.clone(), svg_data);
                }
            }
            arrow_paths.set(new_paths);
        });
    };

    rsx! {
        svg {
            class: "absolute top-0 left-0 float-left w-full h-full pointer-events-auto overflow-visible",
            onmounted: on_mounted,
            {arrow_paths.read().iter().map(|(edge_id, svg_data)| {
                rsx! {
                    g {
                        key: "{edge_id}",
                        path {
                            d: "{svg_data.path}",
                            fill: "none",
                            stroke: "black",
                            "stroke-width": "2"
                        }
                        polygon {
                            points: "-6,-3 0,0 -6,3",
                            fill: "black",
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
