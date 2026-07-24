use std::collections::{HashMap, HashSet};

pub struct MethodDependencyGraph {
    pub cache_validation: HashMap<String, bool>,
    pub cache_dependency_graph: HashMap<String, Vec<String>>,
}


impl MethodDependencyGraph {
    pub fn new() -> Self {
        MethodDependencyGraph{
            cache_validation: HashMap::new(),
            cache_dependency_graph: HashMap::new(),  // maps child_name to Vec<parents_name>
        }
    }

    pub fn add_dependency(&mut self, current: String, dependencies: Vec<String>) -> Option<()> {
        if self.cache_validation.contains_key(&current) {
            return None;
        }
        self.cache_validation.insert(current.clone(), false);
        for dependent_method in dependencies {
            self.cache_dependency_graph
                .entry(dependent_method.clone())
                .or_insert_with(Vec::new)
                .push(current.clone());
            if !self.cache_validation.contains_key(&dependent_method) {
                self.cache_validation
                    .insert(dependent_method.clone(), false);
            }
        }
        return Some(())
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

        Some(())
    }

    pub fn is_valid(&self, current: String) -> bool {
        self.cache_validation.get(&current).copied().unwrap_or(false)
    }

    pub fn validate(&mut self, current: String) -> Option<bool> {
        self.cache_validation.insert(current, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid() {
        let mut dg = MethodDependencyGraph::new();
        dg.add_dependency("A".to_string(), vec!["B".to_string(), "C".to_string()]);
    }
}
