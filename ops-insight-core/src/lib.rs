pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

pub mod config;
pub mod factory;
pub mod helpers;

// DTOs
pub use application::dtos::{IssueDto, QueryRange, ReportDto, SuggestionDto};

// Use case + range helpers
pub use application::use_cases::generate_report::{
    GenerateReportUseCase, daily_range, weekly_range,
};

// Config types
pub use config::{
    AnalyzerConfig, ClaudeConfig, Config, NewRelicConfig, OpenAiConfig, OutputConfig, ServerConfig,
};

// Factory functions
pub use factory::{build_analyzer, load_config};

// Range helpers
pub use helpers::{init_config, parse_custom_range, serilog_range};

// Domain ports (traits)
pub use domain::ports::{Analyzer, DataSource, ReportWriter};
pub use domain::value_objects::SecretKey;

// Infrastructure implementations
pub use infrastructure::claude::ClaudeAnalyzer;
pub use infrastructure::local::LocalAnalyzer;
pub use infrastructure::newrelic::NewRelicSource;
pub use infrastructure::openai::OpenAiAnalyzer;
pub use infrastructure::output::{MarkdownWriter, TerminalWriter};
pub use infrastructure::serilog::SerilogFileSource;
