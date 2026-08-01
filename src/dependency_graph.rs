use std::collections::{HashMap, HashSet};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationState {
    Valid,
    Invalid,
    PermanentlyInvalid,
}


pub struct MethodDependencyGraph {
    validation_state: HashMap<String, ValidationState>,
    dependency_graph: HashMap<String, HashSet<String>>, // maps child_name to set[parents_name], for ease of traversal
}

impl MethodDependencyGraph {
    pub fn new() -> Self {
        return MethodDependencyGraph {
            validation_state: HashMap::new(),
            dependency_graph: HashMap::new(),
        };
    }

    pub fn get_method_state_as_enum(&self, method: &str) -> &ValidationState {
        return self.validation_state.get(method).unwrap_or(&ValidationState::PermanentlyInvalid);
    }

    pub fn list_child_methods(&self, method: &str) -> Vec<String> {
        self.dependency_graph
            .iter()
            .filter(|(_, parents)| parents.contains(method))
            .map(|(child, _)| child.clone())
            .collect()
    }

    pub fn add_children_dependency(&mut self, method: String, dependencies: Vec<String>) -> () {
        if self.validation_state.contains_key(&method) {
            return ();
        }
        self.validation_state.insert(method.clone(), ValidationState::Invalid);
        self.dependency_graph.entry(method.clone()).or_default();
        for dependent_method in dependencies {
            self.dependency_graph
                .entry(dependent_method.clone())
                .or_default()
                .insert(method.clone());
        }
        return ();
    }

    pub fn add_parent_dependency(&mut self, method: String, dependencies: Vec<String>) -> () {
        self.dependency_graph.entry(method.clone()).or_default();
        for dependent_method in dependencies {
            self.dependency_graph
                .entry(dependent_method.clone())
                .or_default()
                .insert(method.clone());
        }
        return ();
    }

    pub fn methods_to_invalidate(&self, method: String) -> Vec<String> {
        let mut queue = Vec::new();
        let mut visited = HashSet::new();
        let mut to_invalidate = Vec::new();

        queue.push(method.clone());
        visited.insert(method.clone());

        while let Some(current_method) = queue.pop() {
            to_invalidate.push(current_method.clone());

            if let Some(parent_methods) = self.dependency_graph.get(&current_method) {
                for parent_method in parent_methods {
                    if visited.insert(parent_method.clone()) {
                        queue.push(parent_method.clone());
                    }
                }
            }
        }
        return to_invalidate;
    }

    pub fn temporarily_invalidate(&mut self, method: String) -> () {
        let to_invalidate = self.methods_to_invalidate(method);

        for invalid_methods in to_invalidate {
            let validity = self.validation_state.get(&invalid_methods).unwrap_or(&ValidationState::PermanentlyInvalid);
            match validity {
                ValidationState::Valid => {
                    self.validation_state.entry(invalid_methods).and_modify(|s| *s = ValidationState::Invalid);
                },
                ValidationState::Invalid => (),
                ValidationState::PermanentlyInvalid => (),
            }
        }

        return ();
    }

    pub fn is_valid(&self, method: String) -> bool {
        let validity = self.validation_state
            .get(&method)
            .unwrap_or(&ValidationState::PermanentlyInvalid);
        return match validity {
            ValidationState::Valid => true,
            ValidationState::Invalid => false,
            ValidationState::PermanentlyInvalid => false,
        };
    }

    pub fn validate(&mut self, method: String) -> () {
        let validity = self.validation_state
            .get(&method)
            .unwrap_or(&ValidationState::PermanentlyInvalid);
        match validity {
            ValidationState::Valid => (),
            ValidationState::Invalid => {
                self.validation_state.entry(method).and_modify(|s| *s = ValidationState::Valid);
            },
            ValidationState::PermanentlyInvalid => (),
        }
        return ();
    }

    pub fn permanently_invalidate(&mut self, method: String) -> () {
        self.validation_state
            .entry(method)
            .and_modify(|state| *state = ValidationState::PermanentlyInvalid)
            .or_insert(ValidationState::PermanentlyInvalid);
        return ();
    }

    pub fn clone_graph(&self) -> HashMap<String, HashSet<String>> {
        return self.dependency_graph
            .clone();
    }

    pub fn clone_state(&self) -> HashMap<String, String> {
        return self.validation_state
            .iter()
            .map(|(k, v)| match v {
                ValidationState::Valid => (k.clone(), "valid".to_string()),
                ValidationState::Invalid => (k.clone(), "invalid".to_string()),
                ValidationState::PermanentlyInvalid => (k.clone(), "permanently invalid".to_string()),
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_state_add_one() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("B".to_string(), vec![]);
        dg.add_children_dependency("C".to_string(), vec![]);

        let mut cache_state = HashMap::new();
        cache_state.insert("A".to_string(), ValidationState::Invalid);
        cache_state.insert("B".to_string(), ValidationState::Invalid);
        cache_state.insert("C".to_string(), ValidationState::Invalid);
        assert_eq!(dg.validation_state, cache_state);
    }

    #[test]
    fn test_cache_state_add_many() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("B".to_string(), vec!["D".to_string(), "E".to_string()]);
        dg.add_children_dependency("C".to_string(), vec!["F".to_string(), "G".to_string()]);
        dg.add_children_dependency("D".to_string(), vec![]);
        dg.add_children_dependency("E".to_string(), vec![]);
        dg.add_children_dependency("F".to_string(), vec![]);
        dg.add_children_dependency("G".to_string(), vec![]);

        let mut cache_state = HashMap::new();
        cache_state.insert("A".to_string(), ValidationState::Invalid);
        cache_state.insert("B".to_string(), ValidationState::Invalid);
        cache_state.insert("C".to_string(), ValidationState::Invalid);
        cache_state.insert("D".to_string(), ValidationState::Invalid);
        cache_state.insert("E".to_string(), ValidationState::Invalid);
        cache_state.insert("F".to_string(), ValidationState::Invalid);
        cache_state.insert("G".to_string(), ValidationState::Invalid);
        assert_eq!(dg.validation_state, cache_state);
    }

    #[test]
    fn test_graph_state_add_one() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);

        let mut graph_state = HashMap::new();
        graph_state.insert("A".to_string(), HashSet::new());
        graph_state.insert("B".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("C".to_string(), HashSet::from(["A".to_string()]));
        assert_eq!(dg.dependency_graph, graph_state);
    }

    #[test]
    fn test_graph_state_add_many() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("B".to_string(), vec!["D".to_string(), "E".to_string()]);
        dg.add_children_dependency("C".to_string(), vec!["F".to_string(), "G".to_string()]);

        let mut graph_state = HashMap::new();
        graph_state.insert("A".to_string(), HashSet::new());
        graph_state.insert("B".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("C".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("D".to_string(), HashSet::from(["B".to_string()]));
        graph_state.insert("E".to_string(), HashSet::from(["B".to_string()]));
        graph_state.insert("F".to_string(), HashSet::from(["C".to_string()]));
        graph_state.insert("G".to_string(), HashSet::from(["C".to_string()]));
        assert_eq!(dg.dependency_graph, graph_state);
    }

    #[test]
    fn test_name_validation() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("B".to_string(), vec![]);
        dg.add_children_dependency("C".to_string(), vec![]);

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
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.validate("A".to_string());
        dg.validate("B".to_string());
        dg.validate("C".to_string());

        dg.temporarily_invalidate("A".to_string());
        assert!(!dg.is_valid("A".to_string()));
        dg.validate("A".to_string());

        dg.temporarily_invalidate("B".to_string());
        assert!(!dg.is_valid("A".to_string()));
        assert!(!dg.is_valid("B".to_string()));
    }

    #[test]
    fn test_add_dependency_idempotency() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);

        let mut graph_state = HashMap::new();
        graph_state.insert("A".to_string(), HashSet::new());
        graph_state.insert("B".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("C".to_string(), HashSet::from(["A".to_string()]));
        assert_eq!(dg.dependency_graph, graph_state);
    }

    #[test]
    fn test_attempt_to_validate_permanently_invalid_state() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("B".to_string(), vec![]);
        dg.add_children_dependency("C".to_string(), vec![]);

        dg.permanently_invalidate("B".to_string());

        dg.validate("B".to_string());

        let mut cache_state = HashMap::new();
        cache_state.insert("A".to_string(), ValidationState::Invalid);
        cache_state.insert("B".to_string(), ValidationState::PermanentlyInvalid);
        cache_state.insert("C".to_string(), ValidationState::Invalid);
        assert_eq!(dg.validation_state, cache_state)
    }

    #[test]
    fn test_attempt_to_invalidate_permanently_invalid_state() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("B".to_string(), vec![]);
        dg.add_children_dependency("C".to_string(), vec![]);

        dg.permanently_invalidate("B".to_string());

        dg.temporarily_invalidate("B".to_string());

        let mut cache_state = HashMap::new();
        cache_state.insert("A".to_string(), ValidationState::Invalid);
        cache_state.insert("B".to_string(), ValidationState::PermanentlyInvalid);
        cache_state.insert("C".to_string(), ValidationState::Invalid);
        assert_eq!(dg.validation_state, cache_state)
    }

    #[test]
    fn test_clones_default() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("B".to_string(), vec![]);
        dg.add_children_dependency("C".to_string(), vec![]);

        let mut cache_state = HashMap::new();
        cache_state.insert("A".to_string(), "invalid".to_string());
        cache_state.insert("B".to_string(), "invalid".to_string());
        cache_state.insert("C".to_string(), "invalid".to_string());

        assert_eq!(dg.clone_state(), cache_state);

        let mut graph_state = HashMap::new();
        graph_state.insert("A".to_string(), HashSet::new());
        graph_state.insert("B".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("C".to_string(), HashSet::from(["A".to_string()]));

        assert_eq!(dg.clone_graph(), graph_state);
    }

    #[test]
    fn test_clones_after_mutation() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        dg.add_children_dependency("B".to_string(), vec![]);
        dg.add_children_dependency("C".to_string(), vec![]);
        dg.validate("A".to_string());
        dg.permanently_invalidate("B".to_string());

        let mut cache_state = HashMap::new();
        cache_state.insert("A".to_string(), "valid".to_string());
        cache_state.insert("B".to_string(), "permanently invalid".to_string());
        cache_state.insert("C".to_string(), "invalid".to_string());

        assert_eq!(dg.clone_state(), cache_state);

        dg.add_parent_dependency("B".to_string(), vec!["D".to_string()]);

        let mut graph_state = HashMap::new();
        graph_state.insert("A".to_string(), HashSet::new());
        graph_state.insert("B".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("C".to_string(), HashSet::from(["A".to_string()]));
        graph_state.insert("D".to_string(), HashSet::from(["B".to_string()]));
        assert_eq!(dg.clone_graph(), graph_state);
    }
}
