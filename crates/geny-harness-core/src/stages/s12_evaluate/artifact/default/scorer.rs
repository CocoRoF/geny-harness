//! Quality scorer implementations.

use serde_json::Value;

use crate::core::stage::Strategy;
use crate::stages::s12_evaluate::interface::QualityScorer;

// ── NoScorer ──

/// Always returns 1.0 — effectively disables scoring.
pub struct NoScorer;

impl NoScorer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for NoScorer {
    fn name(&self) -> &str {
        "no_scorer"
    }

    fn description(&self) -> &str {
        "Always returns 1.0 — scoring disabled"
    }
}

impl QualityScorer for NoScorer {
    fn score(&self, _response: &Value, _metadata: &Value) -> f64 {
        1.0
    }
}

// ── WeightedScorer ──

/// Computes a weighted average from score fields in metadata.
pub struct WeightedScorer {
    pub weights: Vec<(String, f64)>,
}

impl WeightedScorer {
    pub fn new(weights: Vec<(String, f64)>) -> Self {
        Self { weights }
    }
}

impl Default for WeightedScorer {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Strategy for WeightedScorer {
    fn name(&self) -> &str {
        "weighted_scorer"
    }

    fn description(&self) -> &str {
        "Computes weighted average from metadata score fields"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(weights) = config.get("weights").and_then(|v| v.as_object()) {
            self.weights = weights
                .iter()
                .filter_map(|(k, v)| v.as_f64().map(|w| (k.clone(), w)))
                .collect();
        }
    }
}

impl QualityScorer for WeightedScorer {
    fn score(&self, _response: &Value, metadata: &Value) -> f64 {
        if self.weights.is_empty() {
            return 1.0;
        }

        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        for (field, weight) in &self.weights {
            if let Some(score) = metadata.get(field).and_then(|v| v.as_f64()) {
                weighted_sum += score * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            (weighted_sum / total_weight).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_scorer() {
        let scorer = NoScorer::new();
        assert_eq!(scorer.score(&Value::Null, &Value::Null), 1.0);
    }

    #[test]
    fn test_weighted_scorer_empty() {
        let scorer = WeightedScorer::new(Vec::new());
        assert_eq!(scorer.score(&Value::Null, &Value::Null), 1.0);
    }

    #[test]
    fn test_weighted_scorer_with_data() {
        let scorer = WeightedScorer::new(vec![
            ("relevance".to_string(), 0.6),
            ("clarity".to_string(), 0.4),
        ]);
        let metadata = serde_json::json!({
            "relevance": 0.9,
            "clarity": 0.7,
        });
        let score = scorer.score(&Value::Null, &metadata);
        // (0.9*0.6 + 0.7*0.4) / (0.6+0.4) = (0.54+0.28)/1.0 = 0.82
        assert!((score - 0.82).abs() < 0.01);
    }

    #[test]
    fn test_weighted_scorer_partial_data() {
        let scorer = WeightedScorer::new(vec![
            ("relevance".to_string(), 0.6),
            ("clarity".to_string(), 0.4),
        ]);
        let metadata = serde_json::json!({
            "relevance": 0.8,
        });
        let score = scorer.score(&Value::Null, &metadata);
        // Only relevance present: (0.8*0.6) / 0.6 = 0.8
        assert!((score - 0.8).abs() < 0.01);
    }
}
