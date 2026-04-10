//! EventBus — pub/sub for pipeline events.

use log::warn;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::events::types::PipelineEvent;

/// Unique handler ID for deduplication.
static NEXT_HANDLER_ID: AtomicUsize = AtomicUsize::new(0);

/// Handler can be sync or async.
#[derive(Clone)]
pub enum EventHandler {
    Sync(Arc<dyn Fn(&PipelineEvent) + Send + Sync>),
    Async(
        Arc<dyn Fn(PipelineEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    ),
}

struct RegisteredHandler {
    id: usize,
    handler: EventHandler,
}

/// Pipeline event bus — all stage transitions and API events flow through here.
///
/// Supports:
///   - Exact type matching: bus.on("stage.enter", handler)
///   - Wildcard matching: bus.on("*", handler) — receives all events
///   - Prefix matching: bus.on("stage.*", handler) — matches stage.enter, stage.exit, etc.
pub struct EventBus {
    handlers: Arc<Mutex<HashMap<String, Vec<RegisteredHandler>>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a handler. Returns an unsubscribe function.
    pub fn on(&self, event_type: &str, handler: EventHandler) -> Box<dyn Fn() + Send + Sync> {
        let handler_id = NEXT_HANDLER_ID.fetch_add(1, Ordering::Relaxed);
        let registered = RegisteredHandler {
            id: handler_id,
            handler,
        };

        let mut handlers = self.handlers.lock().unwrap();
        handlers
            .entry(event_type.to_string())
            .or_default()
            .push(registered);

        // Return unsubscribe closure capturing Arc to handlers
        let event_type_owned = event_type.to_string();
        let handlers_arc = Arc::clone(&self.handlers);

        Box::new(move || {
            if let Ok(mut handlers) = handlers_arc.lock() {
                if let Some(list) = handlers.get_mut(&event_type_owned) {
                    list.retain(|h| h.id != handler_id);
                }
            }
        })
    }

    /// Register a sync handler. Convenience wrapper.
    pub fn on_sync(
        &self,
        event_type: &str,
        handler: impl Fn(&PipelineEvent) + Send + Sync + 'static,
    ) -> Box<dyn Fn() + Send + Sync> {
        self.on(event_type, EventHandler::Sync(Arc::new(handler)))
    }

    /// Remove all handlers for an event type.
    pub fn off(&self, event_type: &str) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.remove(event_type);
    }

    /// Emit an event to all matching handlers (deduplicated).
    pub async fn emit(&self, event: &PipelineEvent) {
        let matched = self.collect_matched_handlers(event);

        for (id, handler) in matched {
            let _ = id; // Used for deduplication in collect phase
            match handler {
                EventHandler::Sync(f) => {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        f(event);
                    }));
                    if let Err(e) = result {
                        warn!(
                            "Event handler failed on {}: {:?}",
                            event.event_type, e
                        );
                    }
                }
                EventHandler::Async(f) => {
                    let future = f(event.clone());
                    if let Err(e) = tokio::spawn(future).await {
                        warn!(
                            "Async event handler failed on {}: {:?}",
                            event.event_type, e
                        );
                    }
                }
            }
        }
    }

    /// Remove all handlers.
    pub fn clear(&self) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.clear();
    }

    /// Collect matched handlers with deduplication.
    fn collect_matched_handlers(&self, event: &PipelineEvent) -> Vec<(usize, EventHandler)> {
        let handlers = self.handlers.lock().unwrap();
        let mut seen_ids: HashSet<usize> = HashSet::new();
        let mut matched: Vec<(usize, EventHandler)> = Vec::new();

        let mut collect = |handler_list: &[RegisteredHandler]| {
            for h in handler_list {
                if seen_ids.insert(h.id) {
                    matched.push((h.id, h.handler.clone()));
                }
            }
        };

        // Exact match
        if let Some(list) = handlers.get(&event.event_type) {
            collect(list);
        }

        // Wildcard match
        if let Some(list) = handlers.get("*") {
            collect(list);
        }

        // Prefix match (e.g., "stage.*" matches "stage.enter")
        if let Some(pos) = event.event_type.rfind('.') {
            let prefix = format!("{}.*", &event.event_type[..pos]);
            if let Some(list) = handlers.get(&prefix) {
                collect(list);
            }
        }

        matched
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handlers = self.handlers.lock().unwrap();
        let count: usize = handlers.values().map(|v| v.len()).sum();
        f.debug_struct("EventBus")
            .field("handler_count", &count)
            .field("event_types", &handlers.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[tokio::test]
    async fn test_exact_match() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        bus.on_sync("stage.enter", move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        bus.emit(&PipelineEvent::new("stage.enter")).await;
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        // Should NOT match other events
        bus.emit(&PipelineEvent::new("stage.exit")).await;
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_wildcard_match() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        bus.on_sync("*", move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        bus.emit(&PipelineEvent::new("stage.enter")).await;
        bus.emit(&PipelineEvent::new("pipeline.start")).await;
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_prefix_match() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        bus.on_sync("stage.*", move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        bus.emit(&PipelineEvent::new("stage.enter")).await;
        bus.emit(&PipelineEvent::new("stage.exit")).await;
        bus.emit(&PipelineEvent::new("pipeline.start")).await;
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_deduplication() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));

        // Register same event type with wildcard — handler should only fire once
        let c1 = counter.clone();
        bus.on_sync("stage.enter", move |_| {
            c1.fetch_add(1, Ordering::Relaxed);
        });

        bus.emit(&PipelineEvent::new("stage.enter")).await;
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let unsub = bus.on_sync("test", move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        bus.emit(&PipelineEvent::new("test")).await;
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        unsub();
        bus.emit(&PipelineEvent::new("test")).await;
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        bus.on_sync("test", move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        bus.clear();
        bus.emit(&PipelineEvent::new("test")).await;
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }
}
