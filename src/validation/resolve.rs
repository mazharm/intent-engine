//! Reference resolution phase

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::model::{codes, IntentKind, StructuredLocation};
use crate::parser::IntentStore;

use super::ValidationResult;
use super::typecheck::is_native_or_engine_type;

/// Resolved reference graph
#[derive(Debug, Default)]
pub struct ResolvedGraph {
    /// Map from intent ID to its resolved dependencies
    pub dependencies: HashMap<Uuid, Vec<Uuid>>,

    /// Map from intent ID to intents that depend on it
    pub dependents: HashMap<Uuid, Vec<Uuid>>,
}

/// Resolve all references in the intent store
pub fn resolve_references(store: &IntentStore) -> (ResolvedGraph, ValidationResult) {
    let mut result = ValidationResult::new();
    let mut graph = ResolvedGraph::default();

    for doc in store.iter() {
        let mut deps = Vec::new();

        // Resolve type references
        for type_name in doc.get_type_references() {
            // Skip native/engine types (allowed for self-hosting meta intents)
            if is_native_or_engine_type(&type_name) {
                continue;
            }
            match store.get_by_kind_name(IntentKind::Type, &type_name) {
                Some(type_doc) => {
                    deps.push(type_doc.id);
                }
                None => {
                    result.add_error(
                        codes::E005_UNKNOWN_REFERENCE,
                        format!("Unknown type reference: {}", type_name),
                        Some(StructuredLocation {
                            file: doc.source_file.clone().unwrap_or_default(),
                            path: "$.spec".to_string(),
                        }),
                    );
                }
            }
        }

        // Resolve workflow references
        if let Some(workflow_name) = doc.get_workflow_reference() {
            match store.get_by_kind_name(IntentKind::Workflow, &workflow_name) {
                Some(workflow_doc) => {
                    deps.push(workflow_doc.id);
                }
                None => {
                    result.add_error(
                        codes::E005_UNKNOWN_REFERENCE,
                        format!("Unknown workflow reference: {}", workflow_name),
                        Some(StructuredLocation {
                            file: doc.source_file.clone().unwrap_or_default(),
                            path: "$.spec.workflow".to_string(),
                        }),
                    );
                }
            }
        }

        // Resolve service references
        for service_name in doc.get_service_references() {
            match store.get_by_kind_name(IntentKind::Service, &service_name) {
                Some(service_doc) => {
                    deps.push(service_doc.id);
                }
                None => {
                    result.add_error(
                        codes::E005_UNKNOWN_REFERENCE,
                        format!("Unknown service reference: {}", service_name),
                        Some(StructuredLocation {
                            file: doc.source_file.clone().unwrap_or_default(),
                            path: "$.spec".to_string(),
                        }),
                    );
                }
            }
        }

        // Update graph
        for dep_id in &deps {
            graph.dependents.entry(*dep_id).or_default().push(doc.id);
        }
        graph.dependencies.insert(doc.id, deps);
    }

    // Check for circular references
    let circular = detect_circular_references(&graph);
    for cycle in circular {
        let cycle_names: Vec<String> = cycle
            .iter()
            .filter_map(|id| store.get(id).map(|d| d.name.clone()))
            .collect();

        result.add_error(
            codes::E006_CIRCULAR_REFERENCE,
            format!("Circular reference detected: {}", cycle_names.join(" -> ")),
            None,
        );
    }

    (graph, result)
}

/// Detect circular references in the dependency graph
fn detect_circular_references(graph: &ResolvedGraph) -> Vec<Vec<Uuid>> {
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();

    for id in graph.dependencies.keys() {
        if !visited.contains(id) {
            detect_cycle_dfs(
                *id,
                graph,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
            );
        }
    }

    cycles
}

fn detect_cycle_dfs(
    node: Uuid,
    graph: &ResolvedGraph,
    visited: &mut HashSet<Uuid>,
    rec_stack: &mut HashSet<Uuid>,
    path: &mut Vec<Uuid>,
    cycles: &mut Vec<Vec<Uuid>>,
) {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node);

    if let Some(deps) = graph.dependencies.get(&node) {
        for dep in deps {
            if !visited.contains(dep) {
                detect_cycle_dfs(*dep, graph, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(dep) {
                // Found a cycle
                let cycle_start = path.iter().position(|&x| x == *dep).unwrap();
                let cycle: Vec<Uuid> = path[cycle_start..].to_vec();
                cycles.push(cycle);
            }
        }
    }

    path.pop();
    rec_stack.remove(&node);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_store() {
        let store = IntentStore::new();
        let (graph, result) = resolve_references(&store);

        assert!(result.is_valid());
        assert!(graph.dependencies.is_empty());
    }
}
