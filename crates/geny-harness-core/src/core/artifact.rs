//! Pluggable stage implementations system — artifact registry.

use std::collections::HashMap;

/// Stages package path (for reference).
pub const STAGES_PACKAGE: &str = "geny_harness.stages";
pub const ARTIFACT_DIR: &str = "artifact";
pub const DEFAULT_ARTIFACT: &str = "default";

/// Order → module name mapping.
pub fn stage_modules() -> HashMap<u32, &'static str> {
    HashMap::from([
        (1, "s01_input"),
        (2, "s02_context"),
        (3, "s03_system"),
        (4, "s04_guard"),
        (5, "s05_cache"),
        (6, "s06_api"),
        (7, "s07_token"),
        (8, "s08_think"),
        (9, "s09_parse"),
        (10, "s10_tool"),
        (11, "s11_agent"),
        (12, "s12_evaluate"),
        (13, "s13_loop"),
        (14, "s14_emit"),
        (15, "s15_memory"),
        (16, "s16_yield"),
    ])
}

/// Short name aliases.
pub fn stage_aliases() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("input", "s01_input"),
        ("context", "s02_context"),
        ("system", "s03_system"),
        ("guard", "s04_guard"),
        ("cache", "s05_cache"),
        ("api", "s06_api"),
        ("token", "s07_token"),
        ("think", "s08_think"),
        ("parse", "s09_parse"),
        ("tool", "s10_tool"),
        ("agent", "s11_agent"),
        ("evaluate", "s12_evaluate"),
        ("loop", "s13_loop"),
        ("emit", "s14_emit"),
        ("memory", "s15_memory"),
        ("yield", "s16_yield"),
    ])
}

/// Resolve a stage identifier to a module name.
///
/// Accepts: "s01_input", "input", "1", or 1
pub fn resolve_stage_module(stage: &str) -> Result<String, String> {
    let modules = stage_modules();
    let aliases = stage_aliases();

    // Try as module name directly (e.g., "s01_input")
    for module_name in modules.values() {
        if stage == *module_name {
            return Ok(stage.to_string());
        }
    }

    // Try as alias (e.g., "input")
    if let Some(module) = aliases.get(stage) {
        return Ok(module.to_string());
    }

    // Try as number (e.g., "1" or "01")
    if let Ok(order) = stage.parse::<u32>() {
        if let Some(module) = modules.get(&order) {
            return Ok(module.to_string());
        }
    }

    Err(format!("Unknown stage identifier: {}", stage))
}

/// Build complete stage→artifact mapping.
/// Start with "default" for all, then apply overrides.
pub fn get_artifact_map(overrides: Option<&HashMap<String, String>>) -> HashMap<String, String> {
    let modules = stage_modules();
    let mut map: HashMap<String, String> = modules
        .values()
        .map(|m| (m.to_string(), DEFAULT_ARTIFACT.to_string()))
        .collect();

    if let Some(overrides) = overrides {
        for (stage, artifact) in overrides {
            if let Ok(module) = resolve_stage_module(stage) {
                map.insert(module, artifact.clone());
            }
        }
    }

    map
}

/// List available artifacts for a stage.
/// In the Rust version, this returns the compile-time registered artifacts.
pub fn list_artifacts(_stage: &str) -> Vec<String> {
    // In the Rust version, all artifacts are compiled in.
    // Currently only "default" is available.
    vec![DEFAULT_ARTIFACT.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_module_name() {
        assert_eq!(resolve_stage_module("s01_input").unwrap(), "s01_input");
        assert_eq!(resolve_stage_module("s06_api").unwrap(), "s06_api");
    }

    #[test]
    fn test_resolve_alias() {
        assert_eq!(resolve_stage_module("input").unwrap(), "s01_input");
        assert_eq!(resolve_stage_module("api").unwrap(), "s06_api");
        assert_eq!(resolve_stage_module("yield").unwrap(), "s16_yield");
    }

    #[test]
    fn test_resolve_number() {
        assert_eq!(resolve_stage_module("1").unwrap(), "s01_input");
        assert_eq!(resolve_stage_module("6").unwrap(), "s06_api");
        assert_eq!(resolve_stage_module("16").unwrap(), "s16_yield");
    }

    #[test]
    fn test_resolve_unknown() {
        assert!(resolve_stage_module("unknown").is_err());
        assert!(resolve_stage_module("17").is_err());
    }

    #[test]
    fn test_get_artifact_map_defaults() {
        let map = get_artifact_map(None);
        assert_eq!(map.len(), 16);
        assert_eq!(map["s01_input"], "default");
        assert_eq!(map["s16_yield"], "default");
    }

    #[test]
    fn test_get_artifact_map_overrides() {
        let mut overrides = HashMap::new();
        overrides.insert("api".to_string(), "openai".to_string());
        overrides.insert("memory".to_string(), "vector".to_string());

        let map = get_artifact_map(Some(&overrides));
        assert_eq!(map["s06_api"], "openai");
        assert_eq!(map["s15_memory"], "vector");
        assert_eq!(map["s01_input"], "default"); // unchanged
    }
}
