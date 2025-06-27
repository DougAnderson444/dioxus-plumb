use crate::graph_data::EdgeData;
use crate::perfect_arrows::{get_box_to_box_arrow, ArrowOptions, Pos2, Vec2};
use dioxus::logger::tracing;
use dioxus::prelude::*;
use std::{collections::HashMap, f64::consts::PI};
use wasm_bindgen::{prelude::*, JsCast};

#[derive(Clone, Debug, PartialEq)]
struct Rect {
    top: f64,
    right: f64,
    bottom: f64,
    left: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Debug)]
struct EdgeSvgData {
    path: String,
    arrow_transform: String,
    label_x: f64,
    label_y: f64,
}

/// A simple component wrapper for edge rendering
#[component]
pub fn EdgeRenderer(edge: EdgeData) -> Element {
    let mut svg_data = use_signal(|| None::<EdgeSvgData>);

    // Calculate the arrow path when the component mounts
    let edge_clone = edge.clone();
    spawn(async move {
        // Small delay to ensure elements are rendered
        gloo_timers::future::TimeoutFuture::new(300).await;

        match generate_arrow_path_safe(&edge_clone) {
            Ok(data) => svg_data.set(Some(data)),
            Err(err) => {
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                    "Error calculating edge {}: {}",
                    edge_clone.id, err
                )));
            }
        }
    });
    let svg_data = svg_data.read();
    // If we don't have SVG data yet, render nothing
    if svg_data.is_none() {
        return rsx! { g {} };
    }

    let data = svg_data.as_ref().unwrap();
    let edge_label = edge.label.clone();

    rsx! {
        g {
            key: "{edge.id}",
            path {
                d: "{data.path}",
                fill: "none",
                stroke: "#d1d5db",
                "stroke-width": "4",
                class: "edge",
                style: "transition: stroke 0.2s ease; pointer-events: stroke;",
                "stroke-opacity": "0.5"
            }
            polygon {
                points: "-8,-6 0,0 -8,6",
                fill: "#d1d5db",
                transform: "{data.arrow_transform}",
                class: "arrow",
                style: "transition: fill 0.2s ease; pointer-events: stroke;",
            }

            // Render edge label if present
            if let Some(label) = edge_label {
                rect {
                    x: "{data.label_x - 20.0}",
                    y: "{data.label_y - 10.0}",
                    width: "40",
                    height: "20",
                    rx: "5",
                    ry: "5",
                    fill: "white",
                    opacity: "0.8"
                }
                text {
                    x: "{data.label_x}",
                    y: "{data.label_y}",
                    fill: "#4b5563",
                    "font-size": "12px",
                    "text-anchor": "middle",
                    "dy": "0.3em",
                    "{label}"
                }
            }
        }
    }
}

#[component]
pub fn AllEdgesWithMounted(edges: Vec<EdgeData>) -> Element {
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
                gloo_timers::future::TimeoutFuture::new(190).await;
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
            // Enable pointer events on the SVG container
            style: "pointer-events: all;",
            {arrow_paths.read().iter().map(|(edge_id, svg_data)| {
                // Find the edge to get its label
                let edge_label = edges_ref.read().iter()
                    .find(|e| &e.id == edge_id)
                    .and_then(|e| e.label.clone());
                // Define colors outside of the string interpolation
                let default_color = "#d1d5db";
                let stroke_opacity = "0.5";

                rsx! {
                    g {
                        key: "{edge_id}",
                        path {
                            d: "{svg_data.path}",
                            fill: "none",
                            stroke: default_color,
                            "stroke-width": "4",
                            class: "edge",
                            style: "transition: stroke 0.2s ease; pointer-events: stroke;",
                            "stroke-opacity": stroke_opacity
                        }
                        polygon {
                            points: "-8,-6 0,0 -8,6",
                            fill: default_color,
                            transform: "{svg_data.arrow_transform}",
                            class: "arrow",
                            style: "transition: fill 0.2s ease; pointer-events: stroke;",
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

fn generate_arrow_path_safe(edge: &EdgeData) -> Result<EdgeSvgData, String> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Get node elements
    let source_el = document
        .get_element_by_id(&edge.source)
        .ok_or(format!("Source node not found: {}", edge.source))?;

    let target_el = document
        .get_element_by_id(&edge.target)
        .ok_or(format!("Target node not found: {}", edge.target))?;

    // Get the content container
    let content_el = document
        .get_element_by_id("graph-content")
        .ok_or("Content container not found")?;

    // Get element coordinates
    let source = get_coords(&source_el);
    let target = get_coords(&target_el);
    let content = get_coords(&content_el);

    // Calculate positions relative to the content container
    // This is the key change - we use the content container as the reference
    let x_0 = source.left - content.left;
    let y_0 = source.top - content.top;
    let x_1 = target.left - content.left;
    let y_1 = target.top - content.top;

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

    // Get window scroll position
    let window = web_sys::window().unwrap();
    let page_x_offset = window.page_x_offset().unwrap_or(0.0);
    let page_y_offset = window.page_y_offset().unwrap_or(0.0);

    // Calculate absolute position (relative to document)
    Rect {
        top: rect.top() + page_y_offset,
        right: rect.right() + page_x_offset,
        bottom: rect.bottom() + page_y_offset,
        left: rect.left() + page_x_offset,

        // Add these for more accurate positioning
        width: rect.width(),
        height: rect.height(),
    }
}
