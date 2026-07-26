use std::collections::{HashMap, HashSet};

pub struct MethodDependencyGraph {
    pub cache_validation: HashMap<String, bool>,
    pub cache_dependency_graph: HashMap<String, HashSet<String>>, // maps child_name to Vec<parents_name>
}

impl MethodDependencyGraph {
    pub fn new() -> Self {
        return MethodDependencyGraph {
            cache_validation: HashMap::new(),
            cache_dependency_graph: HashMap::new(),
        };
    }

    pub fn add_dependency(&mut self, current: String, dependencies: Vec<String>) -> Option<()> {
        self.cache_validation.insert(current.clone(), false);
        for dependent_method in dependencies {
            self.cache_dependency_graph
                .entry(dependent_method.clone())
                .or_default()
                .insert(current.clone());
            self.cache_validation
                .entry(dependent_method)
                .or_insert(false);
        }
        return Some(());
    }

    pub fn invalidate(&mut self, current: String) -> Option<()> {
        let mut queue = Vec::new();
        let mut visited = HashSet::new();
        let mut to_invalidate = Vec::new();

        queue.push(current.clone());
        visited.insert(current.clone());

        while let Some(node) = queue.pop() {
            to_invalidate.push(node.clone());

            if let Some(parents) = self.cache_dependency_graph.get(&node) {
                for p in parents {
                    if visited.insert(p.clone()) {
                        queue.push(p.clone());
                    }
                }
            }
        }

        for node in to_invalidate {
            self.cache_validation.insert(node, false);
        }

        return Some(());
    }

    pub fn is_valid(&self, current: String) -> bool {
        return self
            .cache_validation
            .get(&current)
            .copied()
            .unwrap_or(false);
    }

    pub fn validate(&mut self, current: String) -> Option<bool> {
        return self.cache_validation.insert(current, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_state_add_one() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);

        let mut cache_state = HashMap::new();
        cache_state.insert("A".to_string(), false);
        cache_state.insert("B".to_string(), false);
        cache_state.insert("C".to_string(), false);
        assert_eq!(dg.cache_validation, cache_state);
    }

    #[test]
    fn test_cache_state_add_many() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_dependency("B".to_string(), vec!["D".to_string(), "E".to_string()]);
        dg.add_dependency("C".to_string(), vec!["F".to_string(), "G".to_string()]);

        let mut cache_state = HashMap::new();
        cache_state.insert("A".to_string(), false);
        cache_state.insert("B".to_string(), false);
        cache_state.insert("C".to_string(), false);
        cache_state.insert("D".to_string(), false);
        cache_state.insert("E".to_string(), false);
        cache_state.insert("F".to_string(), false);
        cache_state.insert("G".to_string(), false);
        assert_eq!(dg.cache_validation, cache_state);
    }

    #[test]
    fn test_graph_state_add_one() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);

        let mut graph_state = HashMap::new();
        graph_state.insert("B".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("C".to_string(), HashSet::from(["A".to_string()]));
        assert_eq!(dg.cache_dependency_graph, graph_state);
    }

    #[test]
    fn test_graph_state_add_many() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_dependency("B".to_string(), vec!["D".to_string(), "E".to_string()]);
        dg.add_dependency("C".to_string(), vec!["F".to_string(), "G".to_string()]);

        let mut graph_state = HashMap::new();
        graph_state.insert("B".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("C".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("D".to_string(), HashSet::from(["B".to_string()]));
        graph_state.insert("E".to_string(), HashSet::from(["B".to_string()]));
        graph_state.insert("F".to_string(), HashSet::from(["C".to_string()]));
        graph_state.insert("G".to_string(), HashSet::from(["C".to_string()]));
        assert_eq!(dg.cache_dependency_graph, graph_state);
    }

    #[test]
    fn test_name_validation() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);

        assert!(!dg.is_valid("A".to_string()));
        assert!(!dg.is_valid("B".to_string()));
        assert!(!dg.is_valid("C".to_string()));

        dg.validate("A".to_string());

        assert!(dg.is_valid("A".to_string()));
        assert!(!dg.is_valid("B".to_string()));
        assert!(!dg.is_valid("C".to_string()));

        dg.validate("C".to_string());

        assert!(dg.is_valid("A".to_string()));
        assert!(!dg.is_valid("B".to_string()));
        assert!(dg.is_valid("C".to_string()));
    }

    #[test]
    fn test_name_invalidation() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.validate("A".to_string());
        dg.validate("B".to_string());
        dg.validate("C".to_string());

        dg.invalidate("A".to_string());
        assert!(!dg.is_valid("A".to_string()));
        dg.validate("A".to_string());

        dg.invalidate("B".to_string());
        assert!(!dg.is_valid("A".to_string()));
        assert!(!dg.is_valid("B".to_string()));
    }

    #[test]
    fn test_add_dependency_idempotency() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);

        let mut graph_state = HashMap::new();
        graph_state.insert("B".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("C".to_string(), HashSet::from(["A".to_string()]));
        assert_eq!(dg.cache_dependency_graph, graph_state);
    }
}
