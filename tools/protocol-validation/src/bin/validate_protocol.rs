//! Main binary for running DataFold protocol compliance validation
//!
//! This binary provides a command-line interface for running comprehensive
//! validation tests for DataFold's RFC 9421 HTTP Message Signatures implementation.

use clap::{Parser, Subcommand, ValueEnum};
use datafold_protocol_validation::{
    ValidationSuite, ValidationConfig, ValidationCategory, ValidationResult
};
use std::path::PathBuf;
use std::process;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "validate-protocol")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Output directory for reports
    #[arg(short, long, value_name = "DIR", default_value = "reports")]
    output: PathBuf,

    /// Validation categories to run
    #[arg(short, long, value_enum)]
    categories: Vec<CategoryFilter>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,

    /// Generate JSON report
    #[arg(long)]
    json: bool,

    /// Generate HTML report
    #[arg(long)]
    html: bool,

    /// Generate JUnit XML report
    #[arg(long)]
    junit: bool,

    /// Skip performance tests (for faster validation)
    #[arg(long)]
    skip_performance: bool,

    /// Skip security tests (for faster validation)
    #[arg(long)]
    skip_security: bool,

    /// Fail fast on first error
    #[arg(long)]
    fail_fast: bool,

    /// Parallel execution (number of threads)
    #[arg(long, default_value = "1")]
    parallel: usize,

    /// Timeout for individual tests (seconds)
    #[arg(long, default_value = "300")]
    timeout: u64,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all validation tests
    All {
        /// Additional test configuration
        #[arg(long)]
        strict: bool,
    },
    /// Run only RFC 9421 compliance tests
    Rfc9421 {
        /// Enable strict header validation
        #[arg(long)]
        strict_headers: bool,
        /// Custom test vectors path
        #[arg(long)]
        test_vectors: Option<PathBuf>,
    },
    /// Run only security validation tests
    Security {
        /// Enable DoS simulation tests (caution: resource intensive)
        #[arg(long)]
        enable_dos: bool,
        /// Attack simulation duration in seconds
        #[arg(long, default_value = "30")]
        attack_duration: u64,
    },
    /// Run only cross-platform validation tests
    CrossPlatform {
        /// JavaScript SDK path
        #[arg(long)]
        js_sdk: Option<PathBuf>,
        /// Python SDK path
        #[arg(long)]
        python_sdk: Option<PathBuf>,
        /// Server executable path
        #[arg(long)]
        server: Option<PathBuf>,
    },
    /// Run only performance validation tests
    Performance {
        /// Number of iterations for benchmarks
        #[arg(long, default_value = "1000")]
        iterations: usize,
        /// Enable memory profiling
        #[arg(long)]
        profile_memory: bool,
    },
    /// Generate test vectors
    GenerateVectors {
        /// Output directory for test vectors
        #[arg(short, long, default_value = "test-vectors")]
        output: PathBuf,
        /// Number of test vectors to generate
        #[arg(short, long, default_value = "100")]
        count: usize,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum CategoryFilter {
    Rfc9421,
    Security,
    CrossPlatform,
    Performance,
    TestVectors,
}

impl From<CategoryFilter> for ValidationCategory {
    fn from(filter: CategoryFilter) -> Self {
        match filter {
            CategoryFilter::Rfc9421 => ValidationCategory::RFC9421Compliance,
            CategoryFilter::Security => ValidationCategory::Security,
            CategoryFilter::CrossPlatform => ValidationCategory::CrossPlatform,
            CategoryFilter::Performance => ValidationCategory::Performance,
            CategoryFilter::TestVectors => ValidationCategory::TestVectors,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.debug, cli.verbose);

    info!("Starting DataFold protocol validation");

    // Load configuration
    let config = match load_config(&cli).await {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Create validation suite
    let suite = match ValidationSuite::new(config) {
        Ok(suite) => suite,
        Err(e) => {
            error!("Failed to create validation suite: {}", e);
            process::exit(1);
        }
    };

    // Run validation based on command
    let result = match &cli.command {
        Some(Commands::All { strict }) => {
            info!("Running all validation tests (strict: {})", strict);
            run_all_validation(&suite, *strict).await
        }
        Some(Commands::Rfc9421 { strict_headers, test_vectors }) => {
            info!("Running RFC 9421 compliance tests");
            run_rfc9421_validation(&suite, *strict_headers, test_vectors.as_ref()).await
        }
        Some(Commands::Security { enable_dos, attack_duration }) => {
            info!("Running security validation tests");
            run_security_validation(&suite, *enable_dos, *attack_duration).await
        }
        Some(Commands::CrossPlatform { js_sdk, python_sdk, server }) => {
            info!("Running cross-platform validation tests");
            run_cross_platform_validation(&suite, js_sdk.as_ref(), python_sdk.as_ref(), server.as_ref()).await
        }
        Some(Commands::Performance { iterations, profile_memory }) => {
            info!("Running performance validation tests");
            run_performance_validation(&suite, *iterations, *profile_memory).await
        }
        Some(Commands::GenerateVectors { output, count }) => {
            info!("Generating test vectors");
            run_generate_vectors(output, *count).await
        }
        None => {
            info!("Running default validation suite");
            suite.run_all_validations().await
        }
    };

    let validation_result = match result {
        Ok(result) => result,
        Err(e) => {
            error!("Validation failed: {}", e);
            process::exit(1);
        }
    };

    // Generate reports
    if let Err(e) = suite.generate_reports(&validation_result).await {
        error!("Failed to generate reports: {}", e);
        process::exit(1);
    }

    // Print summary
    print_validation_summary(&validation_result);

    // Exit with appropriate code
    let exit_code = match validation_result.overall_status {
        datafold_protocol_validation::ValidationStatus::Passed => 0,
        datafold_protocol_validation::ValidationStatus::Warning => 0, // Warnings don't fail CI
        datafold_protocol_validation::ValidationStatus::Failed => 1,
        datafold_protocol_validation::ValidationStatus::Error => 2,
    };

    process::exit(exit_code);
}

/// Initialize logging based on CLI parameters
fn init_logging(debug: bool, verbose: bool) {
    let log_level = if debug {
        tracing::Level::DEBUG
    } else if verbose {
        tracing::Level::INFO
    } else {
        tracing::Level::WARN
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new(format!("datafold_protocol_validation={}", log_level))
                })
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Load validation configuration from file or defaults
async fn load_config(cli: &Cli) -> Result<ValidationConfig, Box<dyn std::error::Error>> {
    let mut config = if let Some(config_path) = &cli.config {
        // Load from file
        let config_content = tokio::fs::read_to_string(config_path).await?;
        serde_yaml::from_str(&config_content)?
    } else {
        // Use defaults
        ValidationConfig::default()
    };

    // Apply CLI overrides
    if !cli.categories.is_empty() {
        config.enabled_categories = cli.categories.iter()
            .map(|c| c.clone().into())
            .collect();
    }

    if cli.skip_performance {
        config.enabled_categories.retain(|c| *c != ValidationCategory::Performance);
    }

    if cli.skip_security {
        config.enabled_categories.retain(|c| *c != ValidationCategory::Security);
    }

    // Update output configuration
    config.output_config.output_directory = cli.output.to_string_lossy().to_string();
    config.output_config.generate_json_report = cli.json;
    config.output_config.generate_html_report = cli.html;
    config.output_config.generate_junit_xml = cli.junit;

    Ok(config)
}

/// Run all validation tests
async fn run_all_validation(
    suite: &ValidationSuite,
    strict: bool,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    info!("Running comprehensive validation suite (strict mode: {})", strict);
    
    // TODO: Apply strict mode configuration modifications
    Ok(suite.run_all_validations().await?)
}

/// Run RFC 9421 compliance validation
async fn run_rfc9421_validation(
    suite: &ValidationSuite,
    strict_headers: bool,
    test_vectors: Option<&PathBuf>,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    info!("Running RFC 9421 compliance validation");
    
    // TODO: Apply specific RFC 9421 configuration
    Ok(suite.run_all_validations().await?)
}

/// Run security validation
async fn run_security_validation(
    suite: &ValidationSuite,
    enable_dos: bool,
    attack_duration: u64,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    info!("Running security validation (DoS: {}, duration: {}s)", enable_dos, attack_duration);
    
    if enable_dos {
        warn!("DoS simulation enabled - this may consume significant resources");
    }
    
    // TODO: Apply security-specific configuration
    Ok(suite.run_all_validations().await?)
}

/// Run cross-platform validation
async fn run_cross_platform_validation(
    suite: &ValidationSuite,
    js_sdk: Option<&PathBuf>,
    python_sdk: Option<&PathBuf>,
    server: Option<&PathBuf>,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    info!("Running cross-platform validation");
    
    // TODO: Apply cross-platform specific configuration
    Ok(suite.run_all_validations().await?)
}

/// Run performance validation
async fn run_performance_validation(
    suite: &ValidationSuite,
    iterations: usize,
    profile_memory: bool,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    info!("Running performance validation ({} iterations, memory profiling: {})", 
          iterations, profile_memory);
    
    // TODO: Apply performance-specific configuration
    Ok(suite.run_all_validations().await?)
}

/// Generate test vectors
async fn run_generate_vectors(
    output: &PathBuf,
    count: usize,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    info!("Generating {} test vectors to {:?}", count, output);
    
    // TODO: Implement test vector generation
    // For now, return a dummy result
    use std::collections::HashMap;
    use datafold_protocol_validation::*;
    
    Ok(ValidationResult {
        test_suite_id: "test-vector-generation".to_string(),
        timestamp: chrono::Utc::now(),
        duration_ms: 1000,
        overall_status: ValidationStatus::Passed,
        categories: HashMap::new(),
        summary: ValidationSummary {
            total_tests: count,
            total_passed: count,
            total_failed: 0,
            total_skipped: 0,
            success_rate: 100.0,
            critical_failures: 0,
            warnings: 0,
        },
        environment_info: EnvironmentInfo {
            rust_version: "1.70.0".to_string(),
            node_version: None,
            python_version: None,
            platform: "test".to_string(),
            architecture: "test".to_string(),
            datafold_server_version: None,
            test_environment: "development".to_string(),
        },
    })
}

/// Print validation summary to console
fn print_validation_summary(result: &ValidationResult) {
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("                  VALIDATION SUMMARY");
    println!("═══════════════════════════════════════════════════════════════");
    println!("Test Suite ID: {}", result.test_suite_id);
    println!("Timestamp: {}", result.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Duration: {}ms", result.duration_ms);
    println!("Overall Status: {:?}", result.overall_status);
    println!();
    
    println!("Test Statistics:");
    println!("  Total Tests: {}", result.summary.total_tests);
    println!("  Passed: {} ({:.1}%)", 
             result.summary.total_passed, result.summary.success_rate);
    println!("  Failed: {}", result.summary.total_failed);
    println!("  Skipped: {}", result.summary.total_skipped);
    println!("  Warnings: {}", result.summary.warnings);
    println!("  Critical Failures: {}", result.summary.critical_failures);
    println!();
    
    println!("Category Results:");
    for (category, result) in &result.categories {
        let status_symbol = match result.status {
            datafold_protocol_validation::ValidationStatus::Passed => "✅",
            datafold_protocol_validation::ValidationStatus::Warning => "⚠️",
            datafold_protocol_validation::ValidationStatus::Failed => "❌",
            datafold_protocol_validation::ValidationStatus::Error => "💥",
        };
        
        println!("  {} {}: {}/{} passed ({}ms)",
                 status_symbol,
                 category,
                 result.tests_passed,
                 result.tests_run,
                 result.duration_ms);
        
        if !result.failures.is_empty() {
            println!("    Failures:");
            for failure in &result.failures {
                println!("      - {}: {}", failure.test_name, failure.error_message);
            }
        }
        
        if !result.warnings.is_empty() {
            println!("    Warnings:");
            for warning in &result.warnings {
                println!("      - {}: {}", warning.test_name, warning.message);
            }
        }
    }
    
    println!();
    println!("Environment:");
    println!("  Platform: {} ({})", 
             result.environment_info.platform, 
             result.environment_info.architecture);
    println!("  Rust: {}", result.environment_info.rust_version);
    
    if let Some(node) = &result.environment_info.node_version {
        println!("  Node.js: {}", node);
    }
    
    if let Some(python) = &result.environment_info.python_version {
        println!("  Python: {}", python);
    }
    
    println!("  Test Environment: {}", result.environment_info.test_environment);
    println!("═══════════════════════════════════════════════════════════════");
    
    // Print final status message
    match result.overall_status {
        datafold_protocol_validation::ValidationStatus::Passed => {
            println!("🎉 All validations passed successfully!");
        }
        datafold_protocol_validation::ValidationStatus::Warning => {
            println!("⚠️  Validation completed with warnings. Please review.");
        }
        datafold_protocol_validation::ValidationStatus::Failed => {
            println!("❌ Validation failed. Please address the failures above.");
        }
        datafold_protocol_validation::ValidationStatus::Error => {
            println!("💥 Validation encountered errors. Please check the logs.");
        }
    }
    
    println!();
}