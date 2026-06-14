//! `tracing` subscriber integration for `Consola`.

// This module is conditionally compiled when feature = "tracing".
// See `super::tracing_impl` declaration in mod.rs.

use crate::constants::LogType;
use crate::types::{ErrorInfo, LogObject};

use super::Consola;

struct ConsolaVisitor<'a> {
    message: Option<String>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> tracing::field::Visit for ConsolaVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        }
    }
}

/// Collects all field values on a span (unlike [`ConsolaVisitor`] which
/// extracts only the single `message` field from an event).
struct SpanFieldCollector {
    fields: Vec<(String, String)>,
}

impl tracing::field::Visit for SpanFieldCollector {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields
            .push((field.name().to_string(), format!("{:?}", value)));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }
}

impl tracing::Subscriber for Consola {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        let level = match *metadata.level() {
            tracing::Level::ERROR => 0,
            tracing::Level::WARN => 1,
            tracing::Level::INFO => 3,
            tracing::Level::DEBUG => 4,
            tracing::Level::TRACE => 5,
        };
        level <= self.level()
    }

    fn max_level_hint(&self) -> Option<tracing::metadata::LevelFilter> {
        let raw = self.level();
        // Negative levels (e.g. SILENT = i32::MIN) mean no events pass;
        // see `enabled()` which compares `level <= raw`.
        if raw < 0 {
            return Some(tracing::metadata::LevelFilter::OFF);
        }
        let filter = match raw.min(5) {
            0 => tracing::metadata::LevelFilter::ERROR,
            1 => tracing::metadata::LevelFilter::WARN,
            2 | 3 => tracing::metadata::LevelFilter::INFO,
            4 => tracing::metadata::LevelFilter::DEBUG,
            _ => tracing::metadata::LevelFilter::TRACE,
        };
        Some(filter)
    }

    fn new_span(&self, attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        let mut state = self.state.lock();
        state.span_id_counter += 1;
        let id = state.span_id_counter;
        state.span_ref_counts.insert(id, 1);
        state.span_metas.insert(id, attrs.metadata());

        // Record initial field values from the span macro.
        let mut collector = SpanFieldCollector { fields: Vec::new() };
        attrs.record(&mut collector);
        if !collector.fields.is_empty() {
            state.span_fields.insert(id, collector.fields);
        }

        tracing::span::Id::from_u64(id)
    }

    fn record(&self, span: &tracing::span::Id, values: &tracing::span::Record<'_>) {
        let mut collector = SpanFieldCollector { fields: Vec::new() };
        values.record(&mut collector);
        if !collector.fields.is_empty() {
            let mut state = self.state.lock();
            state
                .span_fields
                .entry(span.into_u64())
                .or_default()
                .extend(collector.fields);
        }
    }

    fn record_follows_from(&self, span: &tracing::span::Id, follows: &tracing::span::Id) {
        let mut state = self.state.lock();
        state
            .span_follows_from
            .entry(span.into_u64())
            .or_default()
            .push(follows.into_u64());
    }

    fn event(&self, event: &tracing::Event<'_>) {
        let raw_level = match *event.metadata().level() {
            tracing::Level::ERROR => 0,
            tracing::Level::WARN => 1,
            tracing::Level::INFO => 3,
            tracing::Level::DEBUG => 4,
            tracing::Level::TRACE => 5,
        };
        if raw_level > self.level() {
            return;
        }

        let mut visitor = ConsolaVisitor {
            message: None,
            _marker: std::marker::PhantomData,
        };
        event.record(&mut visitor);

        let message = visitor.message.unwrap_or_default();
        let base_tag = event.metadata().target().to_string();

        // Collect current span context (name + recorded fields) without
        // holding the lock across the remaining work.
        let (tag, span_field_args) = {
            let state = self.state.lock();
            let top = state.span_stack.last().copied();
            if let Some(top) = top {
                let span_name = state.span_metas.get(&top).map(|m| m.name().to_string());
                let span_fields = state.span_fields.get(&top).cloned().unwrap_or_default();
                let tag = match span_name {
                    Some(name) => format!("{}::{}", name, base_tag),
                    None => base_tag,
                };
                let args: Vec<String> = span_fields
                    .into_iter()
                    .filter(|(k, _)| k != "message")
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                (tag, args)
            } else {
                (base_tag, Vec::new())
            }
        };

        let mut log_obj = LogObject::new(LogType::Log);
        log_obj.level = raw_level;
        log_obj.r#type = match raw_level {
            0 => LogType::Error,
            1 => LogType::Warn,
            2 | 3 => LogType::Info,
            4 => LogType::Debug,
            _ => LogType::Trace,
        };
        log_obj.tag = tag;
        log_obj.args = if span_field_args.is_empty() {
            vec![message]
        } else {
            let mut args = Vec::with_capacity(span_field_args.len() + 1);
            args.push(message);
            args.extend(span_field_args);
            args
        };

        #[cfg(feature = "backtrace")]
        if raw_level == 0 {
            let bt = backtrace::Backtrace::new();
            log_obj.error = Some(ErrorInfo {
                message: String::new(),
                stack: Some(format!("{:?}", bt)),
                backtrace: Some(format!("{:?}", bt)),
                cause: None,
            });
        }

        self._emit(&log_obj);
    }

    fn enter(&self, span: &tracing::span::Id) {
        let mut state = self.state.lock();
        state.span_stack.push(span.into_u64());
    }

    fn exit(&self, span: &tracing::span::Id) {
        let mut state = self.state.lock();
        // Only pop if the top of the stack matches the exiting span.
        if state.span_stack.last() == Some(&span.into_u64()) {
            state.span_stack.pop();
        }
    }

    fn clone_span(&self, id: &tracing::span::Id) -> tracing::span::Id {
        let mut state = self.state.lock();
        if let Some(count) = state.span_ref_counts.get_mut(&id.into_u64()) {
            *count += 1;
        }
        id.clone()
    }

    fn current_span(&self) -> tracing_core::span::Current {
        let state = self.state.lock();
        let id_raw = state.span_stack.last().copied();
        if let Some(id_raw) = id_raw
            && let Some(&meta) = state.span_metas.get(&id_raw)
        {
            let current = tracing_core::span::Current::new(
                // SAFETY: id_raw was produced by new_span (starts at 1).
                tracing::span::Id::from_u64(id_raw),
                meta,
            );
            drop(state);
            return current;
        }
        drop(state);
        tracing_core::span::Current::none()
    }

    fn try_close(&self, id: tracing::span::Id) -> bool {
        let mut state = self.state.lock();
        let raw = id.into_u64();
        if let Some(count) = state.span_ref_counts.get_mut(&raw) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                state.span_ref_counts.remove(&raw);
                state.span_metas.remove(&raw);
                state.span_fields.remove(&raw);
                state.span_follows_from.remove(&raw);
                // Clean up from stack if still present.
                state.span_stack.retain(|&s| s != raw);
                return true;
            }
        }
        false
    }
}
