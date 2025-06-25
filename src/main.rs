mod perfect_arrows;

use dioxus::logger::tracing;
use dioxus::prelude::*;
use perfect_arrows::{ArrowOptions, get_box_to_box_arrow};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// Each box will have an ID and a ref to access its DOM node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BoxId(&'static str);

/// Represents the position and dimensions of a rectangle on screen
#[derive(Debug, Clone, Copy)]
struct Rect {
    /// X coordinate of the top-left corner
    x: f64,
    /// Y coordinate of the top-left corner
    y: f64,
    /// Width of the rectangle
    width: f64,
    /// Height of the rectangle
    height: f64,
}

impl Rect {
    /// Returns the center point of the rectangle as (x, y) coordinates
    fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    // Track references to each box
    let box_ids = [BoxId("a"), BoxId("b")];
    let refs: HashMap<_, _> = box_ids
        .iter()
        .map(|id| (id.clone(), use_signal(|| Option::<Event<MountedData>>::None)))
        .collect();

    // Store coordinates for each box
    let mut coords = use_signal(HashMap::<BoxId, Rect>::new);

    // Signal to trigger coordinate updates
    let update_trigger = use_signal(|| 0);

    // Effect for handling window resize
    {
        use_effect(move || {
            // Create a resize listener that simply increments the counter
            if let Some(window) = web_sys::window() {
                // Define the handler function
                let mut trigger = update_trigger;
                let mut handler = move || {
                    // Simply increment the counter to trigger a re-render
                    trigger.with_mut(|count| *count += 1);
                };
                
                // Create a static JS function for the resize event
                let mut handler_clone = handler;
                let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    handler_clone();
                }) as Box<dyn FnMut(_)>);
                
                // Register the event
                let _ = window.add_event_listener_with_callback(
                    "resize", 
                    closure.as_ref().unchecked_ref()
                );
                
                // Forget the closure so it doesn't get dropped
                closure.forget();
                
                // Initial trigger
                handler();
            }
            
            // No cleanup needed since we forgot the closure
        });
    }

    // Use a hook to update coordinates whenever the component renders
    use_effect(move || {
        spawn(async move {
            let mut result = HashMap::new();
            
            // Use browser APIs to get elements by ID
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    // Process each box by ID
                    for box_id in &["a", "b"] {
                        let element_id = format!("box-{}", box_id);
                        
                        // Get element by ID
                        if let Some(element) = document.get_element_by_id(&element_id) {
                            // Get bounding rect
                            let rect = element.get_bounding_client_rect();
                            
                            tracing::info!(
                                "DOM element {}: x={}, y={}, w={}, h={}", 
                                box_id, rect.x(), rect.y(), rect.width(), rect.height()
                            );
                            
                            result.insert(
                                BoxId(if *box_id == "a" { "a" } else { "b" }),
                                Rect {
                                    x: rect.x(),
                                    y: rect.y(),
                                    width: rect.width(),
                                    height: rect.height(),
                                },
                            );
                        }
                    }
                }
            }
            
            // Update coordinates when all are collected
            if !result.is_empty() {
                coords.set(result);
            }
        });
    });

    // Create the boxes with their respective positions
    let boxes = box_ids.iter().enumerate().map(|(i, id)| {
        let mut node_ref = refs.get(id).unwrap().clone();
        let positions = [
            "left-1/4 top-1/3",  // Box A
            "left-2/3 top-2/3",  // Box B
        ];
        
        // Simpler onmounted handler
        let mut update_trigger = update_trigger.clone();
        
        rsx! {
            div {
                key: "{id.0}",
                id: "box-{id.0}",
                class: "absolute w-32 h-16 bg-blue-100 border-2 border-blue-500 flex items-center justify-center text-xl font-bold select-none {positions[i]}",
                onmounted: move |element| {
                    node_ref.set(Some(element));
                    // Trigger an update when mounted
                    update_trigger.with_mut(|count| *count += 1);
                },
                "{id.0}"
            }
        }
    });

    // Create arrow points using perfect_arrows
let arrow_path = {
    // Get the coordinates using the original percentage-based approach
    // Box dimensions as percentage of viewport
    let box_width_pct = 8.0;  // w-32 is roughly 8% of viewport
    let box_height_pct = 4.0;  // h-16 is roughly 4% of viewport
    
    // Box A center (25% + half-width, 33% + half-height)
    let x1_pct = 25.0 + (box_width_pct / 2.0);
    let y1_pct = 33.0 + (box_height_pct / 2.0);
    
    // Box B center (66% + half-width, 66% + half-height)
    let x2_pct = 66.0 + (box_width_pct / 2.0);
    let y2_pct = 66.0 + (box_height_pct / 2.0);
    
    // Convert percentages to actual pixel values (for a 100x100 viewBox)
    let start = perfect_arrows::Pos2 {
        x: x1_pct as f32,
        y: y1_pct as f32,
    };
    
    let end = perfect_arrows::Pos2 {
        x: x2_pct as f32,
        y: y2_pct as f32,
    };
    
    // Box sizes in viewBox coordinates
    let box_size = perfect_arrows::Vec2 {
        x: box_width_pct as f32,
        y: box_height_pct as f32,
    };
    
    // Create arrow options
    let options = ArrowOptions {
        bow: 0.2,          // Moderate curve
        stretch: 0.5,      // Moderate stretch
        pad_start: 0.0,    // No padding at start
        pad_end: 0.0,      // No padding at end
        straights: false,  // Force curved arrows
        ..ArrowOptions::default()
    };
    
    // Get arrow points and angles
    let (start_point, control_point, end_point, angle_end, _, _) = 
        get_box_to_box_arrow(start, box_size.clone(), end, box_size.clone(), options);
    
    // Create SVG path for curved arrow
    let path_d = format!(
        "M {},{} Q {},{} {},{}",
        start_point.x, start_point.y,
        control_point.x, control_point.y,
        end_point.x, end_point.y
    );
    
    // Create arrowhead points based on end angle
    let arrow_size = 2.0; // Size of the arrowhead in viewBox coordinates
    let arrow_angle = angle_end as f64;
    
    let arrow_x = end_point.x as f64;
    let arrow_y = end_point.y as f64;
    
    // Calculate arrowhead points
    let angle1 = arrow_angle - 0.5; // Left side of arrowhead
    let angle2 = arrow_angle + 0.5; // Right side of arrowhead
    
    let x1 = arrow_x - arrow_size * angle1.cos();
    let y1 = arrow_y - arrow_size * angle1.sin();
    let x2 = arrow_x - arrow_size * angle2.cos();
    let y2 = arrow_y - arrow_size * angle2.sin();
    
    (path_d, arrow_x, arrow_y, x1, y1, x2, y2)
};

    // Render curved path
    let (path_d, arrow_x, arrow_y, x1, y1, x2, y2) = arrow_path;
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
            
            // Render all boxes
            {boxes}
            
            // Render the curved arrow with perfect_arrows
            svg {
    class: "fixed top-0 left-0 pointer-events-none z-10",
    style: "width: 100vw; height: 100vh; overflow: visible;",
    view_box: "0 0 100 100",
    preserve_aspect_ratio: "none",
    
    path {
        d: "{path_d}",
        fill: "none",
        stroke: "red",
        stroke_width: "0.5"
    }
    // Render arrowhead
    polygon {
        points: "{arrow_x},{arrow_y} {x1},{y1} {x2},{y2}",
        fill: "red"
    }
}        }
    }
}
