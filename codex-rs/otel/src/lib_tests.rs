use super::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn provider_initialization_is_disabled() {
    let settings = OtelSettings {
        environment: "test".to_string(),
        service_name: "codex".to_string(),
        service_version: "0.0.0".to_string(),
        codex_home: PathBuf::new(),
        exporter: OtelExporter::None,
        trace_exporter: OtelExporter::None,
        metrics_exporter: OtelExporter::None,
        runtime_metrics: true,
        span_attributes: BTreeMap::new(),
        tracestate: BTreeMap::new(),
    };

    let provider = OtelProvider::from(&settings).expect("disabled provider should not fail");

    assert!(provider.is_none());
    assert!(global().is_none());
    assert!(global_statsig_metrics_settings().is_none());
}
