use dot_parser::{ast, canonical};
use std::collections::HashMap;

type Att<'a> = (&'a str, &'a str);

/// Owned representation of the graph data
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
