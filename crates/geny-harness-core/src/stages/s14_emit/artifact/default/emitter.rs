//! Emitter implementations.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s14_emit::interface::Emitter;
use crate::stages::s14_emit::types::EmitResult;

// ── TextEmitter ──

/// Emits plain text output via a callback function.
pub struct TextEmitter {
    pub callback: Option<Box<dyn Fn(String) + Send + Sync>>,
}

impl TextEmitter {
    pub fn new(callback: Option<Box<dyn Fn(String) + Send + Sync>>) -> Self {
        Self { callback }
    }
}

impl Strategy for TextEmitter {
    fn name(&self) -> &str {
        "text_emitter"
    }

    fn description(&self) -> &str {
        "Emits plain text output via callback"
    }
}

#[async_trait]
impl Emitter for TextEmitter {
    async fn emit(&self, state: &PipelineState) -> EmitResult {
        let text = &state.final_text;
        if text.is_empty() {
            return EmitResult::new(false);
        }

        if let Some(ref cb) = self.callback {
            cb(text.clone());
        }

        EmitResult::new(true).with_channel("text".to_string())
    }
}

// ── CallbackEmitter ──

/// Emits full pipeline state via a callback.
pub struct CallbackEmitter {
    pub callback: Box<dyn Fn(&PipelineState) + Send + Sync>,
}

impl CallbackEmitter {
    pub fn new(callback: Box<dyn Fn(&PipelineState) + Send + Sync>) -> Self {
        Self { callback }
    }
}

impl Strategy for CallbackEmitter {
    fn name(&self) -> &str {
        "callback_emitter"
    }

    fn description(&self) -> &str {
        "Emits full pipeline state via callback"
    }
}

#[async_trait]
impl Emitter for CallbackEmitter {
    async fn emit(&self, state: &PipelineState) -> EmitResult {
        (self.callback)(state);
        EmitResult::new(true).with_channel("callback".to_string())
    }
}

// ── VTuberEmitter ──

/// Emits avatar state with emotion extraction for VTuber applications.
pub struct VTuberEmitter {
    pub callback: Option<Box<dyn Fn(Value) + Send + Sync>>,
}

impl VTuberEmitter {
    pub fn new(callback: Option<Box<dyn Fn(Value) + Send + Sync>>) -> Self {
        Self { callback }
    }

    /// Extract emotion from text using keyword matching.
    ///
    /// Supports Korean, English, and emoji keywords.
    /// Returns `{ primary, confidence, scores }`.
    fn _extract_emotion(&self, text: &str) -> Value {
        let lower = text.to_lowercase();

        // Keyword sets: (emotion, keywords)
        let emotion_keywords: Vec<(&str, Vec<&str>)> = vec![
            (
                "happy",
                vec![
                    "happy",
                    "glad",
                    "great",
                    "wonderful",
                    "awesome",
                    "excellent",
                    "joy",
                    "pleased",
                    "delighted",
                    "love",
                    // Korean
                    "행복",
                    "기쁘",
                    "좋아",
                    "훌륭",
                    "대단",
                    "사랑",
                    "즐거",
                    // Emoji
                    "😊",
                    "😄",
                    "🎉",
                    "❤️",
                    "💕",
                    "🥰",
                    "😁",
                ],
            ),
            (
                "sad",
                vec![
                    "sad",
                    "sorry",
                    "unfortunate",
                    "disappointed",
                    "regret",
                    "unhappy",
                    "terrible",
                    "awful",
                    // Korean
                    "슬프",
                    "미안",
                    "안타깝",
                    "실망",
                    "후회",
                    "불행",
                    // Emoji
                    "😢",
                    "😭",
                    "💔",
                    "😞",
                    "😥",
                ],
            ),
            (
                "excited",
                vec![
                    "excited",
                    "amazing",
                    "incredible",
                    "fantastic",
                    "wow",
                    "brilliant",
                    "superb",
                    "outstanding",
                    // Korean
                    "신나",
                    "놀라",
                    "대박",
                    "멋지",
                    "환상",
                    "최고",
                    // Emoji
                    "🔥",
                    "✨",
                    "🚀",
                    "💪",
                    "🎊",
                    "⭐",
                ],
            ),
            (
                "thinking",
                vec![
                    "think",
                    "consider",
                    "perhaps",
                    "maybe",
                    "hmm",
                    "interesting",
                    "analyze",
                    "evaluate",
                    "wonder",
                    // Korean
                    "생각",
                    "아마",
                    "흠",
                    "분석",
                    "평가",
                    "궁금",
                    // Emoji
                    "🤔",
                    "💭",
                    "🧐",
                ],
            ),
        ];

        let mut scores: HashMap<&str, f64> = HashMap::new();
        scores.insert("happy", 0.0);
        scores.insert("sad", 0.0);
        scores.insert("excited", 0.0);
        scores.insert("thinking", 0.0);
        scores.insert("neutral", 0.0);

        let mut total_matches = 0usize;

        for (emotion, keywords) in &emotion_keywords {
            let mut count = 0usize;
            for keyword in keywords {
                if lower.contains(keyword) {
                    count += 1;
                }
            }
            if count > 0 {
                *scores.entry(emotion).or_insert(0.0) += count as f64;
                total_matches += count;
            }
        }

        // Determine primary emotion
        let (primary, max_score) = if total_matches == 0 {
            ("neutral", 1.0_f64)
        } else {
            // Normalize scores
            for (_, score) in scores.iter_mut() {
                if total_matches > 0 {
                    *score /= total_matches as f64;
                }
            }
            // Set neutral as inverse of total detected
            *scores.entry("neutral").or_insert(0.0) = if total_matches > 0 { 0.0 } else { 1.0 };

            let mut best = "neutral";
            let mut best_score = 0.0_f64;
            for (&emotion, &score) in &scores {
                if score > best_score {
                    best = emotion;
                    best_score = score;
                }
            }
            (best, best_score)
        };

        let scores_value: serde_json::Map<String, Value> = scores
            .into_iter()
            .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
            .collect();

        serde_json::json!({
            "primary": primary,
            "confidence": max_score,
            "scores": scores_value,
        })
    }
}

impl Strategy for VTuberEmitter {
    fn name(&self) -> &str {
        "vtuber_emitter"
    }

    fn description(&self) -> &str {
        "Extracts emotion and produces avatar state for VTuber rendering"
    }
}

#[async_trait]
impl Emitter for VTuberEmitter {
    async fn emit(&self, state: &PipelineState) -> EmitResult {
        let text = &state.final_text;
        let emotion = self._extract_emotion(text);

        let avatar_state = serde_json::json!({
            "text": text,
            "emotion": emotion,
            "iteration": state.iteration,
            "model": state.model,
        });

        if let Some(ref cb) = self.callback {
            cb(avatar_state.clone());
        }

        EmitResult::new(true)
            .with_channel("vtuber".to_string())
            .with_metadata("emotion".to_string(), emotion)
    }
}

// ── TTSEmitter ──

/// Emits text for text-to-speech processing.
pub struct TTSEmitter {
    pub callback: Option<Box<dyn Fn(String) + Send + Sync>>,
}

impl TTSEmitter {
    pub fn new(callback: Option<Box<dyn Fn(String) + Send + Sync>>) -> Self {
        Self { callback }
    }
}

impl Strategy for TTSEmitter {
    fn name(&self) -> &str {
        "tts_emitter"
    }

    fn description(&self) -> &str {
        "Emits text for TTS processing"
    }
}

#[async_trait]
impl Emitter for TTSEmitter {
    async fn emit(&self, state: &PipelineState) -> EmitResult {
        let text = &state.final_text;
        if text.is_empty() {
            return EmitResult::new(false);
        }

        if let Some(ref cb) = self.callback {
            cb(text.clone());
        }

        EmitResult::new(true)
            .with_channel("tts".to_string())
            .with_metadata("text_length".to_string(), serde_json::json!(text.len()))
    }
}
