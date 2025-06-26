mod perfect_arrows;

use dioxus::prelude::*;
use dot_parser::{
    ast,
    canonical::{self},
};
use perfect_arrows::{get_box_to_box_arrow, ArrowOptions, Pos2, Vec2};
use std::{collections::HashMap, collections::HashSet, f64::consts::PI};
use wasm_bindgen::{prelude::*, JsCast};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

type Att<'a> = (&'a str, &'a str);

/// Owned representation of the graph data
/// So we don't have to deal with the AST lifetimes directly in the components
#[derive(Clone, Debug, PartialEq)]
pub struct GraphData {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
    pub label: Option<String>,
    pub subgraphs: Vec<SubgraphData>,
}

/// Owned representation of the node data
#[derive(Clone, Debug, PartialEq)]
pub struct NodeData {
    pub id: String,
    pub label: Option<String>,
}

/// Owned Edge data
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeData {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
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
pub struct SubgraphData {
    pub id: String,
    pub label: Option<String>,
    pub style: Option<String>,
    pub nodes: Vec<String>,
}

impl GraphData {
    pub fn from_ast(ast_graph: &ast::Graph<Att>) -> Self {
        // Create canonical representation for edges
        let canonical_graph = canonical::Graph::from(ast_graph.clone());

        // Extract graph label
        let label = find_graph_label(&ast_graph.stmts);

        // Extract nodes
        let mut nodes = Vec::new();
        let mut subgraphs = Vec::new();
        let mut node_to_subgraph = HashMap::new();

        // Process all statements to extract nodes and subgraphs
        extract_nodes_and_subgraphs(
            &ast_graph.stmts,
            &mut nodes,
            &mut subgraphs,
            &mut node_to_subgraph,
            None,
        );

        // Extract edges using canonical representation
        let edges = canonical_graph
            .edges
            .set
            .iter()
            .map(|edge| EdgeData {
                id: format!("{}-{}", edge.from, edge.to),
                source: edge.from.clone(),
                target: edge.to.clone(),
                label: edge.attr.elems.iter().find_map(|(k, v)| {
                    if *k == "label" {
                        Some(v.trim_matches('"').to_string())
                    } else {
                        None
                    }
                }),
            })
            .collect();

        GraphData {
            nodes,
            edges,
            label,
            subgraphs,
        }
    }
}

// Find the graph label in statements
fn find_graph_label(stmts: &ast::StmtList<Att>) -> Option<String> {
    for stmt in stmts {
        match stmt {
            ast::Stmt::AttrStmt(ast::AttrStmt::Graph(attr_list)) => {
                for element in &attr_list.elems {
                    for elem in &element.elems {
                        if elem.0 == "label" {
                            return Some(elem.1.trim_matches('"').to_string());
                        }
                    }
                }
            }
            ast::Stmt::IDEq(key, value) => {
                if key == "label" {
                    return Some(value.trim_matches('"').to_string());
                }
            }
            _ => {}
        }
    }
    None
}

// Extract nodes and subgraphs from AST statements
fn extract_nodes_and_subgraphs(
    stmts: &ast::StmtList<Att>,
    nodes: &mut Vec<NodeData>,
    subgraphs: &mut Vec<SubgraphData>,
    node_to_subgraph: &mut HashMap<String, String>,
    parent_subgraph_id: Option<String>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::NodeStmt(node_stmt) => {
                // Extract node info
                let node_id = node_stmt.node.id.clone();
                let node_label = node_stmt.attr.as_ref().and_then(|attr| {
                    attr.clone().flatten().into_iter().find_map(|(key, value)| {
                        if key == "label" {
                            Some(value.trim_matches('"').to_string())
                        } else {
                            None
                        }
                    })
                });

                nodes.push(NodeData {
                    id: node_id.clone(),
                    label: node_label,
                });

                // Associate node with subgraph if we're in one
                if let Some(subgraph_id) = &parent_subgraph_id {
                    node_to_subgraph.insert(node_id.clone(), subgraph_id.clone());

                    // This is key: Add the node to the current subgraph's node list
                    if let Some(subgraph) = subgraphs.iter_mut().find(|s| &s.id == subgraph_id) {
                        subgraph.nodes.push(node_id);
                    }
                }
            }
            ast::Stmt::Subgraph(subgraph) => {
                // Extract subgraph ID
                let subgraph_id = subgraph
                    .id
                    .clone()
                    .unwrap_or_else(|| format!("cluster_{}", subgraphs.len()));

                // Extract subgraph attributes (label, style)
                let mut label = None;
                let mut style = None;

                for sub_stmt in &subgraph.stmts {
                    match sub_stmt {
                        ast::Stmt::IDEq(attr_name, attr_value) => {
                            if attr_name == "label" {
                                label = Some(attr_value.trim_matches('"').to_string());
                            } else if attr_name == "style" {
                                style = Some(attr_value.trim_matches('"').to_string());
                            }
                        }
                        ast::Stmt::AttrStmt(ast::AttrStmt::Graph(attr_list)) => {
                            for element in &attr_list.elems {
                                for elem in &element.elems {
                                    if elem.0 == "label" {
                                        label = Some(elem.1.trim_matches('"').to_string());
                                    } else if elem.0 == "style" {
                                        style = Some(elem.1.trim_matches('"').to_string());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Create the subgraph with empty nodes list
                let sub_data = SubgraphData {
                    id: subgraph_id.clone(),
                    label,
                    style,
                    nodes: Vec::new(),
                };

                subgraphs.push(sub_data);

                // Process the subgraph's contents recursively
                extract_nodes_and_subgraphs(
                    &subgraph.stmts,
                    nodes,
                    subgraphs,
                    node_to_subgraph,
                    Some(subgraph_id),
                );
            }
            _ => {}
        }
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    let dot = r#"digraph G {
        label="Rust Dioxus Graph";  // Name for the whole graph
        subgraph cluster_0 {
            label="Process A";
            style="dashed";
            1 [label="Start"];
            2 [label="Process"];
            3 [label="Decision"];
            1 -> 2 [label="begin"];
            2 -> 3 [label="analyze"];
        }

        subgraph cluster_1 {
            label="Process B";
            style="dashed";
            4 [label="Output"];
            5 [label="End"];
            4 -> 5 [label="finalize"];
        }
        
        3 -> 4 [label="continue"];
        3 -> 5 [label="bypass"];
    }"#
    .to_string();

    // Create graph data from AST
    let graph_data = GraphData::from_ast(&ast::Graph::<Att>::try_from(dot.as_str()).unwrap());

    rsx! {
        Graph { graph_data }
    }
}

#[component]
pub fn Graph(graph_data: GraphData) -> Element {
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

#[component]
fn Canvas(graph_data: GraphData) -> Element {
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

#[component]
fn AllEdgesWithMounted(edges: Vec<EdgeData>) -> Element {
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
        }) as Box<dyn FnMut()>);

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
                // Find the edge to get its label
                let edge_label = edges_ref.read().iter()
                    .find(|e| &e.id == edge_id)
                    .and_then(|e| e.label.clone());

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

                        // Render edge label if present
                        {edge_label.map(|label| {
                            rsx! {
                                rect {
                                    x: "{svg_data.label_x - 20.0}",
                                    y: "{svg_data.label_y - 10.0}",
                                    width: "40",
                                    height: "20",
                                    rx: "5",
                                    ry: "5",
                                    fill: "white",
                                    opacity: "0.8"
                                }
                                text {
                                    x: "{svg_data.label_x}",
                                    y: "{svg_data.label_y}",
                                    fill: "#4b5563",
                                    "font-size": "12px",
                                    "text-anchor": "middle",
                                    "dy": "0.3em",
                                    "{label}"
                                }
                            }
                        })}
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
    label_x: f64,
    label_y: f64,
}

fn generate_arrow_path_safe(edge: &EdgeData) -> Result<EdgeSvgData, String> {
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

    // Calculate midpoint on the curve (t=0.5 on the quadratic bezier)
    let t = 0.5;
    let mt = 1.0 - t;
    let mid_x = mt * mt * sx + 2.0 * mt * t * cx + t * t * ex;
    let mid_y = mt * mt * sy + 2.0 * mt * t * cy + t * t * ey;

    // Calculate tangent vector at midpoint
    let dx_mid = 2.0 * (mt * (cx - sx) + t * (ex - cx));
    let dy_mid = 2.0 * (mt * (cy - sy) + t * (ey - cy));

    // Calculate normal vector (perpendicular to tangent)
    let len = (dx_mid * dx_mid + dy_mid * dy_mid).sqrt();
    let nx = -dy_mid / len;
    let ny = dx_mid / len;

    // Determine which side is the "outside" of the curve
    // We compare the control point position relative to the straight line between start and end
    let center_x = (sx + ex) / 2.0;
    let center_y = (sy + ey) / 2.0;
    let control_side = (cx - center_x) * (ey - sy) - (cy - center_y) * (ex - sx);

    // Adjust normal direction based on the curve's concavity
    // This ensures the label is always on the "outside" of the curve
    let offset = 20.0; // pixels to offset label from curve
    let adjusted_nx = if control_side > 0.0 { nx } else { -nx };
    let adjusted_ny = if control_side > 0.0 { ny } else { -ny };

    // Position the label at midpoint + offset in correct normal direction
    let label_x = mid_x + adjusted_nx * offset;
    let label_y = mid_y + adjusted_ny * offset;

    Ok(EdgeSvgData {
        path,
        arrow_transform,
        label_x,
        label_y,
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
