use codex_config::types::DEFAULT_OTEL_ENVIRONMENT;
use codex_config::types::OtelConfig;
use codex_config::types::OtelConfigToml;
use codex_config::types::OtelExporterKind;
use std::collections::BTreeMap;

pub(crate) fn resolve_config(
    config: OtelConfigToml,
    _startup_warnings: &mut Vec<String>,
) -> OtelConfig {
    let _ = config;

    OtelConfig {
        log_user_prompt: false,
        environment: DEFAULT_OTEL_ENVIRONMENT.to_string(),
        exporter: OtelExporterKind::None,
        trace_exporter: OtelExporterKind::None,
        metrics_exporter: OtelExporterKind::None,
        span_attributes: BTreeMap::new(),
        tracestate: BTreeMap::new(),
    }
}
