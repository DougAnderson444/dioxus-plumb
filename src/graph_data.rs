use dot_parser::{ast, canonical};
use std::collections::HashMap;

type Att<'a> = (&'a str, &'a str);

/// Unified graph structure that can represent both top-level graphs and subgraphs
#[derive(Clone, Debug, PartialEq)]
pub struct GraphData {
    pub id: String,
    pub label: Option<String>,
    pub style: Option<String>,
    pub nodes: Vec<NodeData>,
    pub subgraphs: Vec<GraphData>, // Recursive structure
    pub edges: Vec<EdgeData>,      // Edges within this (sub)graph scope
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

impl GraphData {
    pub fn from_ast(ast_graph: &ast::Graph<Att>) -> Self {
        // Create canonical representation for edges
        let canonical_graph = canonical::Graph::from(ast_graph.clone());

        // Extract graph label and ID
        let label = find_graph_label(&ast_graph.stmts);
        let id = "G".to_string(); // Default graph ID

        // Create a node ID map to track all node IDs across subgraphs
        let mut node_id_map = HashMap::new();

        // Parse the graph recursively
        let mut graph = GraphData {
            id,
            label,
            style: None,
            nodes: Vec::new(),
            subgraphs: Vec::new(),
            edges: Vec::new(),
        };

        // Parse statements to build the graph structure
        parse_statements(&ast_graph.stmts, &mut graph, "", &mut node_id_map);

        // Process edges from canonical representation
        // All edges will be stored only at the top level
        let edge_data: Vec<EdgeData> = canonical_graph
            .edges
            .set
            .iter()
            .map(|edge| {
                // Get the actual node IDs as they exist in our structure
                let source = node_id_map.get(&edge.from).unwrap_or(&edge.from).clone();
                let target = node_id_map.get(&edge.to).unwrap_or(&edge.to).clone();

                EdgeData {
                    id: format!("{}-{}", source, target),
                    source,
                    target,
                    label: edge.attr.elems.iter().find_map(|(k, v)| {
                        if *k == "label" {
                            Some(v.trim_matches('"').to_string())
                        } else {
                            None
                        }
                    }),
                }
            })
            .collect();

        // Store all edges at the top level
        graph.edges = edge_data;

        graph
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

// Parse statements to build the graph structure
fn parse_statements(
    stmts: &ast::StmtList<Att>,
    graph: &mut GraphData,
    path_prefix: &str,
    node_id_map: &mut HashMap<String, String>, // Map of original ID to node ID in our structure
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::NodeStmt(node_stmt) => {
                // Extract node info
                let original_id = node_stmt.node.id.clone();

                // Create node ID with path prefix to ensure uniqueness
                let node_id = if path_prefix.is_empty() {
                    original_id.clone()
                } else {
                    format!("{}-{}", path_prefix, original_id)
                };

                // Map the original ID to our node ID
                node_id_map.insert(original_id.clone(), node_id.clone());

                let node_label = node_stmt.attr.as_ref().and_then(|attr| {
                    attr.clone().flatten().into_iter().find_map(|(key, value)| {
                        if key == "label" {
                            Some(value.trim_matches('"').to_string())
                        } else {
                            None
                        }
                    })
                });

                graph.nodes.push(NodeData {
                    id: node_id,
                    label: node_label,
                });
            }
            ast::Stmt::Subgraph(subgraph) => {
                // Extract subgraph ID
                let subgraph_id = format!("cluster_{}", graph.subgraphs.len());

                // Create unique path prefix for nodes in this subgraph
                let new_path_prefix = if path_prefix.is_empty() {
                    subgraph_id.clone()
                } else {
                    format!("{}-{}", path_prefix, subgraph_id)
                };

                // Extract subgraph attributes
                let mut label = None;
                let mut style = None;
                extract_attributes(&subgraph.stmts, &mut label, &mut style);

                // Create the subgraph
                let mut sub_graph = GraphData {
                    id: subgraph_id,
                    label,
                    style,
                    nodes: Vec::new(),
                    subgraphs: Vec::new(),
                    edges: Vec::new(), // No edges will be stored in subgraphs
                };

                // Recursively parse the subgraph's contents
                parse_statements(
                    &subgraph.stmts,
                    &mut sub_graph,
                    &new_path_prefix,
                    node_id_map,
                );

                // Add the subgraph to the parent graph
                graph.subgraphs.push(sub_graph);
            }
            _ => {}
        }
    }
}

// Helper to extract label and style attributes
fn extract_attributes(
    stmts: &ast::StmtList<Att>,
    label: &mut Option<String>,
    style: &mut Option<String>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::IDEq(attr_name, attr_value) => {
                if attr_name == "label" {
                    *label = Some(attr_value.trim_matches('"').to_string());
                } else if attr_name == "style" {
                    *style = Some(attr_value.trim_matches('"').to_string());
                }
            }
            ast::Stmt::AttrStmt(ast::AttrStmt::Graph(attr_list)) => {
                for element in &attr_list.elems {
                    for elem in &element.elems {
                        if elem.0 == "label" {
                            *label = Some(elem.1.trim_matches('"').to_string());
                        } else if elem.0 == "style" {
                            *style = Some(elem.1.trim_matches('"').to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
