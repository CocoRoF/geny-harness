//! Tool registration and lookup.

use serde_json::Value;
use std::collections::HashSet;

use crate::tools::base::Tool;

/// Central registry for tools.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Register a tool.
    pub fn register(&mut self, tool: Box<dyn Tool>) -> &mut Self {
        self.tools.push(tool);
        self
    }

    /// Remove a tool by name.
    pub fn unregister(&mut self, name: &str) {
        self.tools.retain(|t| t.name() != name);
    }

    /// Lookup by name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }

    /// All registered tools.
    pub fn list_all(&self) -> Vec<&dyn Tool> {
        self.tools.iter().map(|t| t.as_ref()).collect()
    }

    /// All tool names.
    pub fn list_names(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.name()).collect()
    }

    /// Filter tools by include/exclude name sets.
    pub fn filter(
        &self,
        include: Option<&HashSet<String>>,
        exclude: Option<&HashSet<String>>,
    ) -> Vec<&dyn Tool> {
        self.tools
            .iter()
            .filter(|t| {
                if let Some(inc) = include {
                    if !inc.contains(t.name()) {
                        return false;
                    }
                }
                if let Some(exc) = exclude {
                    if exc.contains(t.name()) {
                        return false;
                    }
                }
                true
            })
            .map(|t| t.as_ref())
            .collect()
    }

    /// Export filtered tools in API format.
    pub fn to_api_format(
        &self,
        include: Option<&HashSet<String>>,
        exclude: Option<&HashSet<String>>,
    ) -> Vec<Value> {
        self.filter(include, exclude)
            .iter()
            .map(|t| t.to_api_format())
            .collect()
    }

    /// Tool count.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Check if a tool name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name() == name)
    }
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("count", &self.tools.len())
            .field("names", &self.list_names())
            .finish()
    }
}
