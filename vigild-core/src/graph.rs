use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct DependencyGraph {
    edges: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_edges(&mut self, unit: &str, deps: &[impl AsRef<str>]) {
        self.edges
            .entry(unit.to_string())
            .or_default()
            .extend(deps.iter().map(|d| d.as_ref().to_string()));
    }

    pub fn edges(&self, unit: &str) -> Vec<String> {
        self.edges.get(unit).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_graph_edges() {
        let mut g = DependencyGraph::new();
        g.add_edges("network.target", &["dbus.service", "basic.target"]);
        let edges = g.edges("network.target");
        assert!(edges.contains(&"dbus.service".to_string()));
        assert!(edges.contains(&"basic.target".to_string()));
    }

    #[test]
    fn transitive_empty_for_leaf() {
        let mut g = DependencyGraph::new();
        g.add_edges("leaf.service", &[] as &[&str]);
        assert!(g.edges("leaf.service").is_empty());
    }

    #[test]
    fn missing_node_returns_empty() {
        let g = DependencyGraph::new();
        assert!(g.edges("missing.service").is_empty());
    }
}
