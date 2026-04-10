//! Cost calculator implementations.

use serde_json::Value;
use std::collections::HashMap;

use crate::core::stage::Strategy;
use crate::core::state::TokenUsage;
use crate::stages::s07_token::interface::CostCalculator;

/// Per-million-token pricing for a model.
#[derive(Debug, Clone)]
struct ModelPricing {
    input: f64,
    output: f64,
    cache_write: f64,
    cache_read: f64,
}

// ── AnthropicPricingCalculator ──

/// 2026 Anthropic pricing table (per 1M tokens).
pub struct AnthropicPricingCalculator {
    pricing: HashMap<String, ModelPricing>,
}

impl AnthropicPricingCalculator {
    pub fn new() -> Self {
        let mut pricing = HashMap::new();

        // Claude Opus 4
        pricing.insert(
            "claude-opus-4".to_string(),
            ModelPricing {
                input: 15.0,
                output: 75.0,
                cache_write: 18.75,
                cache_read: 1.50,
            },
        );

        // Claude Sonnet 4
        pricing.insert(
            "claude-sonnet-4".to_string(),
            ModelPricing {
                input: 3.0,
                output: 15.0,
                cache_write: 3.75,
                cache_read: 0.30,
            },
        );

        // Claude Haiku 3.5
        pricing.insert(
            "claude-haiku-3.5".to_string(),
            ModelPricing {
                input: 0.80,
                output: 4.0,
                cache_write: 1.0,
                cache_read: 0.08,
            },
        );

        Self { pricing }
    }

    /// Look up pricing for a model, matching by prefix.
    fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
        // Try exact match first
        if let Some(p) = self.pricing.get(model) {
            return Some(p);
        }
        // Try prefix match (e.g., "claude-sonnet-4-20250514" -> "claude-sonnet-4")
        for (key, pricing) in &self.pricing {
            if model.starts_with(key) {
                return Some(pricing);
            }
        }
        None
    }
}

impl Default for AnthropicPricingCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for AnthropicPricingCalculator {
    fn name(&self) -> &str {
        "anthropic_pricing"
    }

    fn description(&self) -> &str {
        "2026 Anthropic model pricing with cache support"
    }
}

impl CostCalculator for AnthropicPricingCalculator {
    fn calculate(&self, usage: &TokenUsage, model: &str) -> f64 {
        let pricing = match self.get_pricing(model) {
            Some(p) => p,
            None => return 0.0, // Unknown model — no cost
        };

        let per_million = 1_000_000.0;

        let input_cost = usage.input_tokens as f64 * pricing.input / per_million;
        let output_cost = usage.output_tokens as f64 * pricing.output / per_million;
        let cache_write_cost =
            usage.cache_creation_input_tokens as f64 * pricing.cache_write / per_million;
        let cache_read_cost =
            usage.cache_read_input_tokens as f64 * pricing.cache_read / per_million;

        input_cost + output_cost + cache_write_cost + cache_read_cost
    }
}

// ── CustomPricingCalculator ──

/// Flat-rate pricing: fixed input/output rates per million tokens.
pub struct CustomPricingCalculator {
    pub input_rate: f64,
    pub output_rate: f64,
}

impl CustomPricingCalculator {
    pub fn new(input_rate: f64, output_rate: f64) -> Self {
        Self {
            input_rate,
            output_rate,
        }
    }
}

impl Strategy for CustomPricingCalculator {
    fn name(&self) -> &str {
        "custom_pricing"
    }

    fn description(&self) -> &str {
        "Custom flat-rate pricing per million tokens"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(v) = config.get("input_rate").and_then(|v| v.as_f64()) {
            self.input_rate = v;
        }
        if let Some(v) = config.get("output_rate").and_then(|v| v.as_f64()) {
            self.output_rate = v;
        }
    }
}

impl CostCalculator for CustomPricingCalculator {
    fn calculate(&self, usage: &TokenUsage, _model: &str) -> f64 {
        let per_million = 1_000_000.0;
        let input_cost = usage.input_tokens as f64 * self.input_rate / per_million;
        let output_cost = usage.output_tokens as f64 * self.output_rate / per_million;
        input_cost + output_cost
    }
}
