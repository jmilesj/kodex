# codex-otel

In this fork, `codex-otel` is a no-op compatibility crate.

The public API is kept so workspace crates can continue to compile against
types such as `OtelProvider`, `SessionTelemetry`, `MetricsClient`, and the
trace-context helpers. Provider initialization always returns `None`, metrics
methods validate inputs but do not record or export anything, and trace-context
helpers do not extract, inject, or propagate trace headers.

Do not add exporter, analytics, Statsig, OTLP, or OpenTelemetry integration code
to this crate. Fork-only behavior is tracked under `docs/fork-tracking/`.
