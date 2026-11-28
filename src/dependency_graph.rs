use std::collections::{HashMap, HashSet};
use crate::metadata_hash_manager::CacheKey;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationState {
    Valid,
    Invalid,
    PermanentlyInvalid,
}


#[derive(Clone)]
pub struct MethodDependencyGraph {
    validation_state: HashMap<CacheKey, ValidationState>,
    dependency_graph: Vec<HashSet<CacheKey>>,  // maps child_name to set[parents_name], for ease of traversal
}

impl MethodDependencyGraph {
    pub fn new() -> Self {
        return MethodDependencyGraph {
            validation_state: HashMap::new(),
            dependency_graph: Vec::new(),
        };
    }

    pub fn get_method_state_as_enum(&self, key: &CacheKey) -> &ValidationState {
        return self.validation_state.get(key).unwrap_or(&ValidationState::PermanentlyInvalid);
    }

    pub fn list_child_methods(&self, key: &CacheKey) -> Vec<CacheKey> {
        return self.dependency_graph
            .iter()
            .enumerate()
            .filter(|(_, parents)| parents.contains(key))
            .map(|(child, _)| child.clone())
            .collect();
    }

    fn get_max_key(&self, key: CacheKey, dependencies: &[CacheKey]) -> CacheKey {
        return dependencies.iter().copied().chain(std::iter::once(key)).max().unwrap_or(key);
    }

    pub fn add_children_dependency(&mut self, key: CacheKey, dependencies: Vec<CacheKey>) -> () {
        if self.validation_state.contains_key(&key) {
            return ();
        }
        self.validation_state.insert(key.clone(), ValidationState::Invalid);

        let max_key = self.get_max_key(key, &dependencies);
        if self.dependency_graph.len() <= max_key {
            self.dependency_graph.resize_with(max_key + 1, HashSet::new);
        }

        for dependent_key in dependencies {
            self.dependency_graph[dependent_key].insert(key.clone());
        }
        return ();
    }

    pub fn add_parent_dependency(&mut self, key: CacheKey, dependencies: Vec<CacheKey>) -> () {
        let max_key = self.get_max_key(key, &dependencies);
        if self.dependency_graph.len() <= max_key {
            self.dependency_graph.resize_with(max_key + 1, HashSet::new);
        }

        for dependent_key in dependencies {
            self.dependency_graph[dependent_key].insert(key.clone());
        }
        return ();
    }

    pub fn methods_to_invalidate(&self, key: CacheKey) -> Vec<CacheKey> {
        let mut queue = Vec::new();
        let mut visited = HashSet::new();
        let mut to_invalidate = Vec::new();

        queue.push(key.clone());
        visited.insert(key.clone());

        while let Some(current_key) = queue.pop() {
            to_invalidate.push(current_key.clone());

            if let Some(parent_keys) = self.dependency_graph.get(current_key) {
                for parent_key in parent_keys {
                    if visited.insert(parent_key.clone()) {
                        queue.push(parent_key.clone());
                    }
                }
            }
        }
        return to_invalidate;
    }

    pub fn temporarily_invalidate(&mut self, key: CacheKey) -> () {
        let to_invalidate = self.methods_to_invalidate(key);

        for invalid_keys in to_invalidate {
            let validity = self.validation_state.get(&invalid_keys).unwrap_or(&ValidationState::PermanentlyInvalid);
            match validity {
                ValidationState::Valid => {
                    self.validation_state.entry(invalid_keys).and_modify(|s| *s = ValidationState::Invalid);
                },
                ValidationState::Invalid => (),
                ValidationState::PermanentlyInvalid => (),
            }
        }

        return ();
    }

    pub fn is_valid(&self, key: CacheKey) -> bool {
        let validity = self.validation_state
            .get(&key)
            .unwrap_or(&ValidationState::PermanentlyInvalid);
        return match validity {
            ValidationState::Valid => true,
            ValidationState::Invalid => false,
            ValidationState::PermanentlyInvalid => false,
        };
    }

    pub fn validate(&mut self, key: CacheKey) -> () {
        let validity = self.validation_state
            .get(&key)
            .unwrap_or(&ValidationState::PermanentlyInvalid);
        match validity {
            ValidationState::Valid => (),
            ValidationState::Invalid => {
                self.validation_state.entry(key).and_modify(|s| *s = ValidationState::Valid);
            },
            ValidationState::PermanentlyInvalid => (),
        }
        return ();
    }

    pub fn permanently_invalidate(&mut self, method: CacheKey) -> () {
        self.validation_state
            .entry(method)
            .and_modify(|state| *state = ValidationState::PermanentlyInvalid)
            .or_insert(ValidationState::PermanentlyInvalid);
        return ();
    }

    pub fn clone_graph(&self) -> HashMap<CacheKey, HashSet<CacheKey>> {
        return self.dependency_graph.iter().cloned().enumerate().collect();
    }

    pub fn clone_state(&self) -> HashMap<CacheKey, String> {
        return self.validation_state
            .iter()
            .map(|(k, v)| match v {
                ValidationState::Valid => (k.clone(), "valid".to_string()),
                ValidationState::Invalid => (k.clone(), "invalid".to_string()),
                ValidationState::PermanentlyInvalid => (k.clone(), "permanently invalid".to_string()),
            })
            .collect();
    }

    pub fn invalidate_all(&mut self) {
        for state in self.validation_state.values_mut() {
            if *state == ValidationState::Valid {
                *state = ValidationState::Invalid;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_state_add_one() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(1, vec![]);
        dg.add_children_dependency(2, vec![]);

        let mut cache_state = HashMap::new();
        cache_state.insert(0, ValidationState::Invalid);
        cache_state.insert(1, ValidationState::Invalid);
        cache_state.insert(2, ValidationState::Invalid);
        assert_eq!(dg.validation_state, cache_state);
    }

    #[test]
    fn test_cache_state_add_many() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(1, vec![3, 4]);
        dg.add_children_dependency(2, vec![5, 6]);
        dg.add_children_dependency(3, vec![]);
        dg.add_children_dependency(4, vec![]);
        dg.add_children_dependency(5, vec![]);
        dg.add_children_dependency(6, vec![]);

        let mut cache_state = HashMap::new();
        cache_state.insert(0, ValidationState::Invalid);
        cache_state.insert(1, ValidationState::Invalid);
        cache_state.insert(2, ValidationState::Invalid);
        cache_state.insert(3, ValidationState::Invalid);
        cache_state.insert(4, ValidationState::Invalid);
        cache_state.insert(5, ValidationState::Invalid);
        cache_state.insert(6, ValidationState::Invalid);
        assert_eq!(dg.validation_state, cache_state);
    }

    #[test]
    fn test_graph_state_add_one() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);

        let mut graph_state = HashMap::new();
        graph_state.insert(0, HashSet::new());
        graph_state.insert(1, HashSet::from([0]));
        graph_state.insert(2, HashSet::from([0]));
        assert_eq!(dg.clone_graph(), graph_state);
    }

    #[test]
    fn test_graph_state_add_many() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(1, vec![3, 4]);
        dg.add_children_dependency(2, vec![5, 6]);

        let mut graph_state = HashMap::new();
        graph_state.insert(0, HashSet::new());
        graph_state.insert(1, HashSet::from([0]));
        graph_state.insert(2, HashSet::from([0]));
        graph_state.insert(3, HashSet::from([1]));
        graph_state.insert(4, HashSet::from([1]));
        graph_state.insert(5, HashSet::from([2]));
        graph_state.insert(6, HashSet::from([2]));
        assert_eq!(dg.clone_graph(), graph_state);
    }

    #[test]
    fn test_name_validation() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(1, vec![]);
        dg.add_children_dependency(2, vec![]);

        assert!(!dg.is_valid(0));
        assert!(!dg.is_valid(1));
        assert!(!dg.is_valid(2));

        dg.validate(0);

        assert!(dg.is_valid(0));
        assert!(!dg.is_valid(1));
        assert!(!dg.is_valid(2));

        dg.validate(2);

        assert!(dg.is_valid(0));
        assert!(!dg.is_valid(1));
        assert!(dg.is_valid(2));
    }

    #[test]
    fn test_name_invalidation() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.validate(0);
        dg.validate(1);
        dg.validate(2);

        dg.temporarily_invalidate(0);
        assert!(!dg.is_valid(0));
        dg.validate(0);

        dg.temporarily_invalidate(1);
        assert!(!dg.is_valid(0));
        assert!(!dg.is_valid(1));
    }

    #[test]
    fn test_add_dependency_idempotency() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(0, vec![1, 2]);

        let mut graph_state = HashMap::new();
        graph_state.insert(0, HashSet::new());
        graph_state.insert(1, HashSet::from([0]));
        graph_state.insert(2, HashSet::from([0]));
        assert_eq!(dg.clone_graph(), graph_state);
    }

    #[test]
    fn test_attempt_to_validate_permanently_invalid_state() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(1, vec![]);
        dg.add_children_dependency(2, vec![]);

        dg.permanently_invalidate(1);

        dg.validate(1);

        let mut cache_state = HashMap::new();
        cache_state.insert(0, ValidationState::Invalid);
        cache_state.insert(1, ValidationState::PermanentlyInvalid);
        cache_state.insert(2, ValidationState::Invalid);
        assert_eq!(dg.validation_state, cache_state)
    }

    #[test]
    fn test_attempt_to_invalidate_permanently_invalid_state() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(1, vec![]);
        dg.add_children_dependency(2, vec![]);

        dg.permanently_invalidate(1);

        dg.temporarily_invalidate(1);

        let mut cache_state = HashMap::new();
        cache_state.insert(0, ValidationState::Invalid);
        cache_state.insert(1, ValidationState::PermanentlyInvalid);
        cache_state.insert(2, ValidationState::Invalid);
        assert_eq!(dg.validation_state, cache_state)
    }

    #[test]
    fn test_clones_default() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(1, vec![]);
        dg.add_children_dependency(2, vec![]);

        let mut cache_state = HashMap::new();
        cache_state.insert(0, "invalid".to_string());
        cache_state.insert(1, "invalid".to_string());
        cache_state.insert(2, "invalid".to_string());

        assert_eq!(dg.clone_state(), cache_state);

        let mut graph_state = HashMap::new();
        graph_state.insert(0, HashSet::new());
        graph_state.insert(1, HashSet::from([0]));
        graph_state.insert(2, HashSet::from([0]));

        assert_eq!(dg.clone_graph(), graph_state);

        let mut metadata_store = HashMap::new();
        metadata_store.insert(0, 0);
        metadata_store.insert(1, 1);
        metadata_store.insert(2, 2);
    }

    #[test]
    fn test_clones_after_mutation() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_children_dependency(0, vec![1, 2]);
        dg.add_children_dependency(1, vec![]);
        dg.add_children_dependency(2, vec![]);
        dg.validate(0);
        dg.permanently_invalidate(1);

        let mut cache_state = HashMap::new();
        cache_state.insert(0, "valid".to_string());
        cache_state.insert(1, "permanently invalid".to_string());
        cache_state.insert(2, "invalid".to_string());

        assert_eq!(dg.clone_state(), cache_state);

        dg.add_children_dependency(3, vec![]);
        dg.add_parent_dependency(1, vec![3]);

        let mut graph_state = HashMap::new();
        graph_state.insert(0, HashSet::new());
        graph_state.insert(1, HashSet::from([0]));
        graph_state.insert(2, HashSet::from([0]));
        graph_state.insert(3, HashSet::from([1]));
        assert_eq!(dg.clone_graph(), graph_state);
    }
}
