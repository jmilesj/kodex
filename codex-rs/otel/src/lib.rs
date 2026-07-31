use codex_protocol::ThreadId;
use codex_protocol::auth::AuthMode;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::W3cTraceContext;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use tracing::Span;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Identity;
use tracing_subscriber::registry::LookupSpan;

pub use codex_utils_string::sanitize_metric_tag_value;

pub mod config {
    use super::*;

    #[derive(Clone, Debug)]
    pub struct OtelSettings {
        pub environment: String,
        pub service_name: String,
        pub service_version: String,
        pub codex_home: PathBuf,
        pub exporter: OtelExporter,
        pub trace_exporter: OtelExporter,
        pub metrics_exporter: OtelExporter,
        pub runtime_metrics: bool,
        pub span_attributes: BTreeMap<String, String>,
        pub tracestate: BTreeMap<String, BTreeMap<String, String>>,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct StatsigMetricsSettings {
        pub environment: String,
    }

    #[derive(Clone, Debug)]
    pub enum OtelHttpProtocol {
        Binary,
        Json,
    }

    #[derive(Clone, Debug, Default)]
    pub struct OtelTlsConfig {
        pub ca_certificate: Option<AbsolutePathBuf>,
        pub client_certificate: Option<AbsolutePathBuf>,
        pub client_private_key: Option<AbsolutePathBuf>,
    }

    #[derive(Clone, Debug)]
    pub enum OtelExporter {
        None,
        Statsig,
        OtlpGrpc {
            endpoint: String,
            headers: HashMap<String, String>,
            tls: Option<OtelTlsConfig>,
        },
        OtlpHttp {
            endpoint: String,
            headers: HashMap<String, String>,
            protocol: OtelHttpProtocol,
            tls: Option<OtelTlsConfig>,
        },
    }

    pub fn validate_span_attributes(attributes: &BTreeMap<String, String>) -> std::io::Result<()> {
        if attributes.keys().any(String::is_empty) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "configured span attribute key must not be empty",
            ));
        }
        Ok(())
    }
}

pub mod metrics {
    use super::*;

    pub const TOOL_CALL_COUNT_METRIC: &str = "codex.tool.call";
    pub const TOOL_CALL_DURATION_METRIC: &str = "codex.tool.call.duration_ms";
    pub const TOOL_CALL_UNIFIED_EXEC_METRIC: &str = "codex.tool.unified_exec";
    pub const PROCESS_START_METRIC: &str = "codex.process.start";
    pub const API_CALL_COUNT_METRIC: &str = "codex.api_request";
    pub const API_CALL_DURATION_METRIC: &str = "codex.api_request.duration_ms";
    pub const SSE_EVENT_COUNT_METRIC: &str = "codex.sse_event";
    pub const SSE_EVENT_DURATION_METRIC: &str = "codex.sse_event.duration_ms";
    pub const WEBSOCKET_REQUEST_COUNT_METRIC: &str = "codex.websocket.request";
    pub const WEBSOCKET_REQUEST_DURATION_METRIC: &str = "codex.websocket.request.duration_ms";
    pub const WEBSOCKET_EVENT_COUNT_METRIC: &str = "codex.websocket.event";
    pub const WEBSOCKET_EVENT_DURATION_METRIC: &str = "codex.websocket.event.duration_ms";
    pub const RESPONSES_API_OVERHEAD_DURATION_METRIC: &str =
        "codex.responses_api_overhead.duration_ms";
    pub const RESPONSES_API_INFERENCE_TIME_DURATION_METRIC: &str =
        "codex.responses_api_inference_time.duration_ms";
    pub const RESPONSES_API_ENGINE_IAPI_TTFT_DURATION_METRIC: &str =
        "codex.responses_api_engine_iapi_ttft.duration_ms";
    pub const RESPONSES_API_ENGINE_SERVICE_TTFT_DURATION_METRIC: &str =
        "codex.responses_api_engine_service_ttft.duration_ms";
    pub const RESPONSES_API_ENGINE_IAPI_TBT_DURATION_METRIC: &str =
        "codex.responses_api_engine_iapi_tbt.duration_ms";
    pub const RESPONSES_API_ENGINE_SERVICE_TBT_DURATION_METRIC: &str =
        "codex.responses_api_engine_service_tbt.duration_ms";
    pub const TURN_E2E_DURATION_METRIC: &str = "codex.turn.e2e_duration_ms";
    pub const TURN_TTFT_DURATION_METRIC: &str = "codex.turn.ttft.duration_ms";
    pub const TURN_TTFM_DURATION_METRIC: &str = "codex.turn.ttfm.duration_ms";
    pub const TURN_NETWORK_PROXY_METRIC: &str = "codex.turn.network_proxy";
    pub const TURN_MEMORY_METRIC: &str = "codex.turn.memory";
    pub const TURN_TOOL_CALL_METRIC: &str = "codex.turn.tool.call";
    pub const TURN_TOKEN_USAGE_METRIC: &str = "codex.turn.token_usage";
    pub const GUARDIAN_REVIEW_COUNT_METRIC: &str = "codex.guardian.review";
    pub const GUARDIAN_REVIEW_DURATION_METRIC: &str = "codex.guardian.review.duration_ms";
    pub const GUARDIAN_REVIEW_TTFT_DURATION_METRIC: &str = "codex.guardian.review.ttft.duration_ms";
    pub const GUARDIAN_REVIEW_TOKEN_USAGE_METRIC: &str = "codex.guardian.review.token_usage";
    pub const GOAL_CREATED_METRIC: &str = "codex.goal.created";
    pub const GOAL_RESUMED_METRIC: &str = "codex.goal.resumed";
    pub const GOAL_COMPLETED_METRIC: &str = "codex.goal.completed";
    pub const GOAL_BUDGET_LIMITED_METRIC: &str = "codex.goal.budget_limited";
    pub const GOAL_BLOCKED_METRIC: &str = "codex.goal.blocked";
    pub const GOAL_TOKEN_COUNT_METRIC: &str = "codex.goal.token_count";
    pub const GOAL_DURATION_SECONDS_METRIC: &str = "codex.goal.duration_s";
    pub const PLUGIN_INSTALL_ELICITATION_SENT_METRIC: &str =
        "codex.plugins.install_elicitation.sent";
    pub const PLUGIN_INSTALL_SUGGESTION_METRIC: &str = "codex.plugins.install_suggestion";
    pub const CURATED_PLUGINS_STARTUP_SYNC_METRIC: &str = "codex.plugins.startup_sync";
    pub const CURATED_PLUGINS_STARTUP_SYNC_FINAL_METRIC: &str = "codex.plugins.startup_sync.final";
    pub const HOOK_RUN_METRIC: &str = "codex.hooks.run";
    pub const HOOK_RUN_DURATION_METRIC: &str = "codex.hooks.run.duration_ms";
    pub const STARTUP_PHASE_DURATION_METRIC: &str = "codex.startup.phase.duration_ms";
    pub const STARTUP_PREWARM_DURATION_METRIC: &str = "codex.startup_prewarm.duration_ms";
    pub const STARTUP_PREWARM_AGE_AT_FIRST_TURN_METRIC: &str =
        "codex.startup_prewarm.age_at_first_turn_ms";
    pub const THREAD_STARTED_METRIC: &str = "codex.thread.started";
    pub const THREAD_SKILLS_ENABLED_TOTAL_METRIC: &str = "codex.thread.skills.enabled_total";
    pub const THREAD_SKILLS_KEPT_TOTAL_METRIC: &str = "codex.thread.skills.kept_total";
    pub const THREAD_SKILLS_DESCRIPTION_TRUNCATED_CHARS_METRIC: &str =
        "codex.thread.skills.description_truncated_chars";
    pub const THREAD_SKILLS_TRUNCATED_METRIC: &str = "codex.thread.skills.truncated";
    pub const ORIGINATOR_TAG: &str = "originator";

    pub type Result<T> = std::result::Result<T, MetricsError>;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum MetricsError {
        EmptyMetricName,
        InvalidMetricName { name: String },
        EmptyTagComponent { label: String },
        InvalidTagComponent { label: String, value: String },
        ExporterDisabled,
        NegativeCounterIncrement { name: String, inc: i64 },
        InvalidConfig { message: String },
        RuntimeSnapshotUnavailable,
    }

    impl fmt::Display for MetricsError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::EmptyMetricName => write!(f, "metric name cannot be empty"),
                Self::InvalidMetricName { name } => {
                    write!(f, "metric name contains invalid characters: {name}")
                }
                Self::EmptyTagComponent { label } => write!(f, "{label} cannot be empty"),
                Self::InvalidTagComponent { label, value } => {
                    write!(f, "{label} contains invalid characters: {value}")
                }
                Self::ExporterDisabled => write!(f, "metrics exporter is disabled"),
                Self::NegativeCounterIncrement { name, inc } => {
                    write!(
                        f,
                        "counter increment must be non-negative for {name}: {inc}"
                    )
                }
                Self::InvalidConfig { message } => {
                    write!(f, "invalid metrics configuration: {message}")
                }
                Self::RuntimeSnapshotUnavailable => {
                    write!(f, "runtime metrics snapshot reader is not enabled")
                }
            }
        }
    }

    impl Error for MetricsError {}

    #[derive(Clone, Debug)]
    pub enum MetricsExporter {
        Disabled,
    }

    #[derive(Clone, Debug)]
    pub struct MetricsConfig {
        default_tags: BTreeMap<String, String>,
    }

    impl MetricsConfig {
        pub fn otlp(
            _environment: impl Into<String>,
            _service_name: impl Into<String>,
            _service_version: impl Into<String>,
            _exporter: crate::OtelExporter,
        ) -> Self {
            Self {
                default_tags: BTreeMap::new(),
            }
        }

        pub fn in_memory<T>(
            _environment: impl Into<String>,
            _service_name: impl Into<String>,
            _service_version: impl Into<String>,
            _exporter: T,
        ) -> Self {
            Self {
                default_tags: BTreeMap::new(),
            }
        }

        pub fn with_export_interval(self, _interval: Duration) -> Self {
            self
        }

        pub fn with_runtime_reader(self) -> Self {
            self
        }

        pub fn with_tag(
            mut self,
            key: impl Into<String>,
            value: impl Into<String>,
        ) -> Result<Self> {
            let key = key.into();
            let value = value.into();
            validate_tag_component("tag key", &key)?;
            validate_tag_component("tag value", &value)?;
            self.default_tags.insert(key, value);
            Ok(self)
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct MetricsClient;

    impl MetricsClient {
        pub fn new(_config: MetricsConfig) -> Result<Self> {
            Ok(Self)
        }

        pub fn counter(&self, name: &str, inc: i64, tags: &[(&str, &str)]) -> Result<()> {
            validate_metric_name(name)?;
            validate_tags(tags)?;
            if inc < 0 {
                return Err(MetricsError::NegativeCounterIncrement {
                    name: name.to_string(),
                    inc,
                });
            }
            Ok(())
        }

        pub fn counter_with_description(
            &self,
            name: &str,
            _description: &str,
            inc: i64,
            tags: &[(&str, &str)],
        ) -> Result<()> {
            self.counter(name, inc, tags)
        }

        pub fn histogram(&self, name: &str, _value: i64, tags: &[(&str, &str)]) -> Result<()> {
            validate_metric_name(name)?;
            validate_tags(tags)
        }

        pub fn gauge(&self, name: &str, _value: i64, tags: &[(&str, &str)]) -> Result<()> {
            validate_metric_name(name)?;
            validate_tags(tags)
        }

        pub fn gauge_with_description(
            &self,
            name: &str,
            _description: &str,
            value: i64,
            tags: &[(&str, &str)],
        ) -> Result<()> {
            self.gauge(name, value, tags)
        }

        pub fn register_observable_gauge_with_description(
            &self,
            name: &str,
            _description: &str,
            _observe: impl Fn() -> i64 + Send + Sync + 'static,
            tags: &[(&str, &str)],
        ) -> Result<()> {
            validate_metric_name(name)?;
            validate_tags(tags)
        }

        pub fn record_duration(
            &self,
            name: &str,
            _duration: Duration,
            tags: &[(&str, &str)],
        ) -> Result<()> {
            validate_metric_name(name)?;
            validate_tags(tags)
        }

        pub fn record_duration_seconds_with_description(
            &self,
            name: &str,
            _description: &str,
            _duration: Duration,
            tags: &[(&str, &str)],
        ) -> Result<()> {
            validate_metric_name(name)?;
            validate_tags(tags)
        }

        pub fn start_timer(&self, name: &str, tags: &[(&str, &str)]) -> Result<Timer> {
            validate_metric_name(name)?;
            validate_tags(tags)?;
            Ok(Timer::new())
        }

        pub fn snapshot(&self) -> Result<MetricsSnapshot> {
            Err(MetricsError::RuntimeSnapshotUnavailable)
        }

        pub fn shutdown(&self) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct MetricsSnapshot;

    #[derive(Debug, Clone)]
    pub struct Timer {
        started_at: Instant,
    }

    impl Timer {
        pub(crate) fn new() -> Self {
            Self {
                started_at: Instant::now(),
            }
        }

        pub fn record(&self, _additional_tags: &[(&str, &str)]) -> Result<()> {
            let _ = self.started_at.elapsed();
            Ok(())
        }
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct RuntimeMetricTotals {
        pub count: u64,
        pub duration_ms: u64,
    }

    impl RuntimeMetricTotals {
        pub fn is_empty(self) -> bool {
            self.count == 0 && self.duration_ms == 0
        }

        pub fn merge(&mut self, other: Self) {
            self.count = self.count.saturating_add(other.count);
            self.duration_ms = self.duration_ms.saturating_add(other.duration_ms);
        }
    }

    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    pub struct RuntimeMetricsSummary {
        pub tool_calls: RuntimeMetricTotals,
        pub api_calls: RuntimeMetricTotals,
        pub streaming_events: RuntimeMetricTotals,
        pub websocket_calls: RuntimeMetricTotals,
        pub websocket_events: RuntimeMetricTotals,
        pub responses_api_overhead_ms: u64,
        pub responses_api_inference_time_ms: u64,
        pub responses_api_engine_iapi_ttft_ms: u64,
        pub responses_api_engine_service_ttft_ms: u64,
        pub responses_api_engine_iapi_tbt_ms: f64,
        pub responses_api_engine_service_tbt_ms: f64,
        pub turn_ttft_ms: u64,
        pub turn_ttfm_ms: u64,
    }

    impl RuntimeMetricsSummary {
        pub fn is_empty(self) -> bool {
            self.tool_calls.is_empty()
                && self.api_calls.is_empty()
                && self.streaming_events.is_empty()
                && self.websocket_calls.is_empty()
                && self.websocket_events.is_empty()
                && self.responses_api_overhead_ms == 0
                && self.responses_api_inference_time_ms == 0
                && self.responses_api_engine_iapi_ttft_ms == 0
                && self.responses_api_engine_service_ttft_ms == 0
                && self.responses_api_engine_iapi_tbt_ms == 0.0
                && self.responses_api_engine_service_tbt_ms == 0.0
                && self.turn_ttft_ms == 0
                && self.turn_ttfm_ms == 0
        }

        pub fn merge(&mut self, other: Self) {
            self.tool_calls.merge(other.tool_calls);
            self.api_calls.merge(other.api_calls);
            self.streaming_events.merge(other.streaming_events);
            self.websocket_calls.merge(other.websocket_calls);
            self.websocket_events.merge(other.websocket_events);
            if other.responses_api_overhead_ms > 0 {
                self.responses_api_overhead_ms = other.responses_api_overhead_ms;
            }
            if other.responses_api_inference_time_ms > 0 {
                self.responses_api_inference_time_ms = other.responses_api_inference_time_ms;
            }
            if other.responses_api_engine_iapi_ttft_ms > 0 {
                self.responses_api_engine_iapi_ttft_ms = other.responses_api_engine_iapi_ttft_ms;
            }
            if other.responses_api_engine_service_ttft_ms > 0 {
                self.responses_api_engine_service_ttft_ms =
                    other.responses_api_engine_service_ttft_ms;
            }
            if other.responses_api_engine_iapi_tbt_ms > 0.0 {
                self.responses_api_engine_iapi_tbt_ms = other.responses_api_engine_iapi_tbt_ms;
            }
            if other.responses_api_engine_service_tbt_ms > 0.0 {
                self.responses_api_engine_service_tbt_ms =
                    other.responses_api_engine_service_tbt_ms;
            }
            if other.turn_ttft_ms > 0 {
                self.turn_ttft_ms = other.turn_ttft_ms;
            }
            if other.turn_ttfm_ms > 0 {
                self.turn_ttfm_ms = other.turn_ttfm_ms;
            }
        }

        pub fn responses_api_summary(&self) -> RuntimeMetricsSummary {
            Self {
                responses_api_overhead_ms: self.responses_api_overhead_ms,
                responses_api_inference_time_ms: self.responses_api_inference_time_ms,
                responses_api_engine_iapi_ttft_ms: self.responses_api_engine_iapi_ttft_ms,
                responses_api_engine_service_ttft_ms: self.responses_api_engine_service_ttft_ms,
                responses_api_engine_iapi_tbt_ms: self.responses_api_engine_iapi_tbt_ms,
                responses_api_engine_service_tbt_ms: self.responses_api_engine_service_tbt_ms,
                ..RuntimeMetricsSummary::default()
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct SessionMetricTagValues {
        pub service_name: Option<String>,
        pub originator: String,
        pub auth_mode: Option<String>,
        pub session_source: String,
        pub model: String,
        pub app_version: &'static str,
    }

    impl SessionMetricTagValues {
        pub fn into_tags(self) -> Vec<(&'static str, String)> {
            let mut tags = vec![
                ("originator", self.originator),
                ("session_source", self.session_source),
                ("model", self.model),
                ("app_version", self.app_version.to_string()),
            ];
            if let Some(service_name) = self.service_name {
                tags.push(("service_name", service_name));
            }
            if let Some(auth_mode) = self.auth_mode {
                tags.push(("auth_mode", auth_mode));
            }
            tags
        }
    }

    pub fn global() -> Option<MetricsClient> {
        None
    }

    pub fn record_process_start_once(_metrics: &MetricsClient, _originator: &str) -> Result<()> {
        Ok(())
    }

    pub fn bounded_originator_tag_value(originator: &str) -> &'static str {
        match originator {
            "codex_cli" | "codex_tui" | "codex_exec" | "codex-exec-server" => "codex",
            _ => "other",
        }
    }

    fn validate_metric_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(MetricsError::EmptyMetricName);
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        {
            return Err(MetricsError::InvalidMetricName {
                name: name.to_string(),
            });
        }
        Ok(())
    }

    fn validate_tags(tags: &[(&str, &str)]) -> Result<()> {
        for (key, value) in tags {
            validate_tag_component("tag key", key)?;
            validate_tag_component("tag value", value)?;
        }
        Ok(())
    }

    fn validate_tag_component(label: &str, value: &str) -> Result<()> {
        if value.is_empty() {
            return Err(MetricsError::EmptyTagComponent {
                label: label.to_string(),
            });
        }
        if value.contains(['\n', '\r']) {
            return Err(MetricsError::InvalidTagComponent {
                label: label.to_string(),
                value: value.to_string(),
            });
        }
        Ok(())
    }

    pub mod runtime_metrics {
        pub use super::RuntimeMetricTotals;
        pub use super::RuntimeMetricsSummary;
    }

    pub mod timer {
        pub use super::Timer;
    }
}

pub mod provider {
    use super::*;

    #[derive(Clone, Debug, Default)]
    pub struct OtelProvider;

    impl OtelProvider {
        pub fn shutdown(&self) {}

        pub fn from(_settings: &OtelSettings) -> std::result::Result<Option<Self>, Box<dyn Error>> {
            Ok(None)
        }

        pub fn logger_layer<S>(&self) -> Option<impl Layer<S> + Send + Sync>
        where
            S: tracing::Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
        {
            None::<Identity>
        }

        pub fn tracing_layer<S>(&self) -> Option<impl Layer<S> + Send + Sync>
        where
            S: tracing::Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
        {
            None::<Identity>
        }

        pub fn codex_export_filter(_meta: &tracing::Metadata<'_>) -> bool {
            false
        }

        pub fn log_export_filter(_meta: &tracing::Metadata<'_>) -> bool {
            false
        }

        pub fn trace_export_filter(_meta: &tracing::Metadata<'_>) -> bool {
            false
        }

        pub fn metrics(&self) -> Option<&MetricsClient> {
            None
        }
    }
}

pub mod trace_context {
    use super::*;

    #[derive(Clone, Debug, Default)]
    pub struct TraceContext;

    pub fn current_span_w3c_trace_context() -> Option<W3cTraceContext> {
        None
    }

    pub fn span_w3c_trace_context(_span: &Span) -> Option<W3cTraceContext> {
        None
    }

    pub fn inject_span_w3c_trace_headers(_span: &Span, _headers: &mut http::HeaderMap) -> bool {
        false
    }

    pub fn current_span_trace_id() -> Option<String> {
        None
    }

    pub fn context_from_w3c_trace_context(_trace: &W3cTraceContext) -> Option<TraceContext> {
        None
    }

    pub fn set_parent_from_w3c_trace_context(_span: &Span, _trace: &W3cTraceContext) -> bool {
        false
    }

    pub fn set_parent_from_context(_span: &Span, _context: TraceContext) {}

    pub fn traceparent_context_from_env() -> Option<TraceContext> {
        None
    }

    pub fn validate_tracestate_entries(
        entries: &BTreeMap<String, BTreeMap<String, String>>,
    ) -> std::result::Result<(), Box<dyn Error>> {
        for (member, fields) in entries {
            validate_tracestate_member(member, fields)?;
        }
        Ok(())
    }

    pub fn validate_tracestate_member(
        member: &str,
        fields: &BTreeMap<String, String>,
    ) -> std::result::Result<(), Box<dyn Error>> {
        if member.trim().is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "tracestate member key must not be empty",
            )));
        }
        if fields.keys().any(String::is_empty) || fields.values().any(String::is_empty) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "tracestate fields must not be empty",
            )));
        }
        Ok(())
    }
}

pub use crate::config::OtelExporter;
pub use crate::config::OtelHttpProtocol;
pub use crate::config::OtelSettings;
pub use crate::config::OtelTlsConfig;
pub use crate::config::StatsigMetricsSettings;
pub use crate::config::validate_span_attributes;
pub use crate::metrics::*;
pub use crate::provider::OtelProvider;
pub use crate::trace_context::TraceContext;
pub use crate::trace_context::context_from_w3c_trace_context;
pub use crate::trace_context::current_span_trace_id;
pub use crate::trace_context::current_span_w3c_trace_context;
pub use crate::trace_context::inject_span_w3c_trace_headers;
pub use crate::trace_context::set_parent_from_context;
pub use crate::trace_context::set_parent_from_w3c_trace_context;
pub use crate::trace_context::span_w3c_trace_context;
pub use crate::trace_context::traceparent_context_from_env;
pub use crate::trace_context::validate_tracestate_entries;
pub use crate::trace_context::validate_tracestate_member;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolDecisionSource {
    AutomatedReviewer,
    Config,
    User,
}

impl fmt::Display for ToolDecisionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AutomatedReviewer => write!(f, "AutomatedReviewer"),
            Self::Config => write!(f, "Config"),
            Self::User => write!(f, "User"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryAuthMode {
    ApiKey,
    Chatgpt,
}

impl fmt::Display for TelemetryAuthMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApiKey => write!(f, "ApiKey"),
            Self::Chatgpt => write!(f, "Chatgpt"),
        }
    }
}

impl From<AuthMode> for TelemetryAuthMode {
    fn from(mode: AuthMode) -> Self {
        match mode {
            AuthMode::ApiKey | AuthMode::BedrockApiKey => Self::ApiKey,
            AuthMode::Chatgpt
            | AuthMode::ChatgptAuthTokens
            | AuthMode::Headers
            | AuthMode::AgentIdentity
            | AuthMode::PersonalAccessToken => Self::Chatgpt,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthEnvTelemetryMetadata {
    pub openai_api_key_env_present: bool,
    pub codex_api_key_env_present: bool,
    pub codex_api_key_env_enabled: bool,
    pub provider_env_key_name: Option<String>,
    pub provider_env_key_present: Option<bool>,
    pub refresh_token_url_override_present: bool,
}

#[derive(Debug, Clone)]
pub struct SessionTelemetry {
    log_user_prompts: bool,
}

impl SessionTelemetry {
    pub fn with_auth_env(self, _auth_env: AuthEnvTelemetryMetadata) -> Self {
        self
    }

    pub fn with_model(self, _model: &str, _slug: &str) -> Self {
        self
    }

    pub fn with_inference_request<E>(
        self,
        _service_tier: Option<&str>,
        _model_reasoning_effort: Option<&E>,
    ) -> Self {
        self
    }

    pub fn with_metrics_service_name(self, _service_name: &str) -> Self {
        self
    }

    pub fn with_metrics(self, _metrics: MetricsClient) -> Self {
        self
    }

    pub fn with_metrics_without_metadata_tags(self, _metrics: MetricsClient) -> Self {
        self
    }

    pub fn with_metrics_config(self, config: MetricsConfig) -> metrics::Result<Self> {
        let _ = MetricsClient::new(config)?;
        Ok(self)
    }

    pub fn with_provider_metrics(self, _provider: &OtelProvider) -> Self {
        self
    }

    pub fn counter(&self, _name: &str, _inc: i64, _tags: &[(&str, &str)]) {}

    pub fn histogram(&self, _name: &str, _value: i64, _tags: &[(&str, &str)]) {}

    pub fn record_duration(&self, _name: &str, _duration: Duration, _tags: &[(&str, &str)]) {}

    pub fn record_startup_phase(
        &self,
        _phase: &'static str,
        _duration: Duration,
        _status: Option<&'static str>,
    ) {
    }

    pub fn record_turn_ttft(&self, _duration: Duration) {}

    pub fn record_plugin_install_elicitation_sent(
        &self,
        _tool_type: &str,
        _tool_id: &str,
        _tool_name: &str,
    ) {
    }

    pub fn record_plugin_install_suggestion(
        &self,
        _tool_type: &str,
        _tool_id: &str,
        _tool_name: &str,
        _response_action: &str,
        _user_confirmed: bool,
        _completed: bool,
    ) {
    }

    pub fn start_timer(&self, _name: &str, _tags: &[(&str, &str)]) -> metrics::Result<Timer> {
        Ok(Timer::new())
    }

    pub fn shutdown_metrics(&self) -> metrics::Result<()> {
        Ok(())
    }

    pub fn snapshot_metrics(&self) -> metrics::Result<MetricsSnapshot> {
        Err(MetricsError::RuntimeSnapshotUnavailable)
    }

    pub fn reset_runtime_metrics(&self) {}

    pub fn runtime_metrics_summary(&self) -> Option<RuntimeMetricsSummary> {
        None
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        _conversation_id: ThreadId,
        _model: &str,
        _slug: &str,
        _account_id: Option<String>,
        _account_email: Option<String>,
        _auth_mode: Option<TelemetryAuthMode>,
        _originator: String,
        log_user_prompts: bool,
        _terminal_type: String,
        _session_source: SessionSource,
    ) -> Self {
        Self { log_user_prompts }
    }

    pub fn record_responses<E>(&self, _handle_responses_span: &Span, _event: &E) {}

    #[allow(clippy::too_many_arguments)]
    pub fn conversation_starts<R, S, A, P>(
        &self,
        _provider_name: &str,
        _reasoning_effort: Option<R>,
        _reasoning_summary: S,
        _context_window: Option<i64>,
        _auto_compact_token_limit: Option<i64>,
        _approval_policy: A,
        _sandbox_policy: P,
        _mcp_servers: Vec<&str>,
    ) {
    }

    pub async fn log_request<F, Fut, T, E>(&self, _attempt: u64, f: F) -> std::result::Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = std::result::Result<T, E>>,
    {
        f().await
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_api_request(
        &self,
        _attempt: u64,
        _status: Option<u16>,
        _error: Option<&str>,
        _duration: Duration,
        _auth_header_attached: bool,
        _auth_header_name: Option<&str>,
        _retry_after_unauthorized: bool,
        _recovery_mode: Option<&str>,
        _recovery_phase: Option<&str>,
        _endpoint: &str,
        _request_id: Option<&str>,
        _cf_ray: Option<&str>,
        _auth_error: Option<&str>,
        _auth_error_code: Option<&str>,
        _agent_identity_telemetry: Option<&impl std::fmt::Debug>,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_websocket_connect(
        &self,
        _duration: Duration,
        _status: Option<u16>,
        _error: Option<&str>,
        _auth_header_attached: bool,
        _auth_header_name: Option<&str>,
        _retry_after_unauthorized: bool,
        _recovery_mode: Option<&str>,
        _recovery_phase: Option<&str>,
        _endpoint: &str,
        _connection_reused: bool,
        _request_id: Option<&str>,
        _cf_ray: Option<&str>,
        _auth_error: Option<&str>,
        _auth_error_code: Option<&str>,
        _agent_identity_telemetry: Option<&impl std::fmt::Debug>,
    ) {
    }

    pub fn record_websocket_request(
        &self,
        _duration: Duration,
        _error: Option<&str>,
        _connection_reused: bool,
        _agent_identity_telemetry: Option<&impl std::fmt::Debug>,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_auth_recovery(
        &self,
        _mode: &str,
        _step: &str,
        _outcome: &str,
        _request_id: Option<&str>,
        _cf_ray: Option<&str>,
        _auth_error: Option<&str>,
        _auth_error_code: Option<&str>,
        _recovery_reason: Option<&str>,
        _auth_state_changed: Option<bool>,
    ) {
    }

    pub fn record_websocket_event<T>(&self, _result: &T, _duration: Duration) {}

    pub fn log_sse_event<T>(&self, _response: &T, _duration: Duration) {}

    pub fn sse_event_failed<T>(&self, _kind: Option<&String>, _duration: Duration, _error: &T)
    where
        T: std::fmt::Display,
    {
    }

    pub fn see_event_completed_failed<T>(&self, _error: &T)
    where
        T: std::fmt::Display,
    {
    }

    pub fn sse_event_completed<T>(&self, _usage: &T, _ttft_ms: Option<i64>) {}

    pub fn user_prompt<T>(&self, _items: &[T]) {
        let _ = self.log_user_prompts;
    }

    pub fn tool_decision<D>(
        &self,
        _tool_name: &str,
        _call_id: &str,
        _decision: &D,
        _source: ToolDecisionSource,
    ) {
    }

    pub fn sandbox_outcome(
        &self,
        _tool_name: &str,
        _call_id: &str,
        _outcome: &str,
        _initial_duration: Duration,
        _escalated_duration: Option<Duration>,
    ) {
    }

    pub async fn log_tool_result_with_tags<F, Fut, E>(
        &self,
        _tool_name: &str,
        _call_id: &str,
        _arguments: &str,
        _extra_tags: &[(&str, &str)],
        _extra_trace_fields: &[(&str, &str)],
        f: F,
    ) -> std::result::Result<(String, bool), E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = std::result::Result<(String, bool), E>>,
    {
        f().await
    }

    pub fn log_tool_failed(&self, _tool_name: &str, _error: &str) {}

    #[allow(clippy::too_many_arguments)]
    pub fn tool_result_with_tags(
        &self,
        _tool_name: &str,
        _call_id: &str,
        _arguments: &str,
        _duration: Duration,
        _success: bool,
        _output: &str,
        _extra_tags: &[(&str, &str)],
        _extra_trace_fields: &[(&str, &str)],
    ) {
    }
}

pub fn start_global_timer(_name: &str, _tags: &[(&str, &str)]) -> metrics::Result<Timer> {
    Ok(Timer::new())
}

pub fn global_statsig_metrics_settings() -> Option<StatsigMetricsSettings> {
    None
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
