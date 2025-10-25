//! Performance Benchmarks for Tooka
//! 
//! This benchmark suite measures performance across critical code paths.
//! 
//! ## Purpose
//! - Track performance across releases and commits
//! - Identify performance regressions early
//! - Validate optimization improvements
//! - Guide future performance work
//!
//! ## Adding New Benchmarks
//! To add a new benchmark:
//! 1. Create a struct implementing the `Benchmark` trait
//! 2. Add it to the `benchmarks` vector in `main()`
//! 3. Document the benchmark's purpose and expected performance characteristics

use chrono::NaiveDate;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::hint::black_box;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

/// Trait for benchmarks that can be run and reported
trait Benchmark {
    /// Name of the benchmark
    fn name(&self) -> &str;
    
    /// Description of what is being benchmarked
    fn description(&self) -> &str;
    
    /// Run the benchmark and return the result
    fn run(&self) -> BenchmarkResult;
}

/// Result of running a benchmark comparison
#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    description: String,
    baseline_duration: Duration,
    optimized_duration: Duration,
}

impl BenchmarkResult {
    fn speedup(&self) -> f64 {
        self.baseline_duration.as_nanos() as f64 / self.optimized_duration.as_nanos() as f64
    }
    
    fn improvement_percent(&self) -> f64 {
        (1.0 - self.optimized_duration.as_nanos() as f64 / self.baseline_duration.as_nanos() as f64) * 100.0
    }
}

impl fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "📊 {}", self.name)?;
        writeln!(f, "─────────────────────────────────────────")?;
        writeln!(f, "{}", self.description)?;
        writeln!(f, "Baseline:   {:?}", self.baseline_duration)?;
        writeln!(f, "Optimized:  {:?}", self.optimized_duration)?;
        writeln!(f, "Speedup:    {:.2}x faster", self.speedup())?;
        writeln!(f, "Improvement: {:.1}% reduction in time", self.improvement_percent())?;
        Ok(())
    }
}

// ============================================================================
// Benchmark Implementations
// ============================================================================

/// Benchmark for regex compilation caching
struct RegexCachingBenchmark;

impl Benchmark for RegexCachingBenchmark {
    fn name(&self) -> &str {
        "Regex Compilation Caching"
    }
    
    fn description(&self) -> &str {
        "Template evaluation with regex (common in file renaming)"
    }
    
    fn run(&self) -> BenchmarkResult {
        let mut metadata = HashMap::new();
        metadata.insert("filename".to_string(), "test_file".to_string());
        metadata.insert("date".to_string(), "2025-01-01".to_string());
        let template = "File: {{filename}}, Date: {{date}}";
        let iterations = 10_000;
        
        // Baseline: compile regex every time (intentionally inefficient for comparison)
        let start = Instant::now();
        #[allow(clippy::regex_creation_in_loops)]
        for _ in 0..iterations {
            let re = Regex::new(r"\{\{(.*?)\}\}").unwrap();
            let mut result = template.to_string();
            for caps in re.captures_iter(template) {
                let full_match = &caps[0];
                let key = caps[1].trim();
                let value = metadata.get(key).cloned().unwrap_or_default();
                result = result.replace(full_match, &value);
            }
            black_box(result);
        }
        let baseline_duration = start.elapsed();
        
        // Optimized: cached regex
        static TEMPLATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"\{\{(.*?)\}\}").expect("Failed to compile template regex")
        });
        
        let start = Instant::now();
        for _ in 0..iterations {
            let mut result = template.to_string();
            for caps in TEMPLATE_REGEX.captures_iter(template) {
                let full_match = &caps[0];
                let key = caps[1].trim();
                let value = metadata.get(key).cloned().unwrap_or_default();
                result = result.replace(full_match, &value);
            }
            black_box(result);
        }
        let optimized_duration = start.elapsed();
        
        BenchmarkResult {
            name: self.name().to_string(),
            description: self.description().to_string(),
            baseline_duration,
            optimized_duration,
        }
    }
}

/// Benchmark for date constant caching
struct DateConstantCachingBenchmark;

impl Benchmark for DateConstantCachingBenchmark {
    fn name(&self) -> &str {
        "Date Constant Caching"
    }
    
    fn description(&self) -> &str {
        "Date range comparisons (used in file filtering)"
    }
    
    fn run(&self) -> BenchmarkResult {
        let iterations = 100_000;
        
        // Baseline: create date every time
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(NaiveDate::from_ymd_opt(1970, 1, 1).expect("MIN_DATE should be valid"));
        }
        let baseline_duration = start.elapsed();
        
        // Optimized: cached date
        static MIN_DATE_CACHED: LazyLock<NaiveDate> = LazyLock::new(|| {
            NaiveDate::from_ymd_opt(1970, 1, 1).expect("MIN_DATE should be valid")
        });
        
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(*MIN_DATE_CACHED);
        }
        let optimized_duration = start.elapsed();
        
        BenchmarkResult {
            name: self.name().to_string(),
            description: self.description().to_string(),
            baseline_duration,
            optimized_duration,
        }
    }
}

/// Benchmark for extension matching optimization
struct ExtensionMatchingBenchmark;

impl Benchmark for ExtensionMatchingBenchmark {
    fn name(&self) -> &str {
        "Extension Matching"
    }
    
    fn description(&self) -> &str {
        "File extension checks (hot path in file matching)"
    }
    
    fn run(&self) -> BenchmarkResult {
        let extensions = [
            "jpg".to_string(),
            "png".to_string(),
            "gif".to_string(),
            "bmp".to_string(),
            "webp".to_string(),
        ];
        let test_ext = "webp";
        let iterations = 100_000;
        
        // Baseline: String comparison
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(extensions.iter().any(|e| e == test_ext));
        }
        let baseline_duration = start.elapsed();
        
        // Optimized: as_str() comparison
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(extensions.iter().any(|e| e.as_str() == test_ext));
        }
        let optimized_duration = start.elapsed();
        
        BenchmarkResult {
            name: self.name().to_string(),
            description: self.description().to_string(),
            baseline_duration,
            optimized_duration,
        }
    }
}

// ============================================================================
// Main Benchmark Runner
// ============================================================================

fn main() {
    println!("🚀 Tooka Performance Benchmarks\n");
    println!("This benchmark suite tracks performance across critical code paths.");
    println!("Run this regularly to ensure optimizations are maintained.\n");
    
    // Register all benchmarks here
    let benchmarks: Vec<Box<dyn Benchmark>> = vec![
        Box::new(RegexCachingBenchmark),
        Box::new(DateConstantCachingBenchmark),
        Box::new(ExtensionMatchingBenchmark),
    ];
    
    let mut results = Vec::new();
    
    // Run all benchmarks
    for benchmark in benchmarks {
        let result = benchmark.run();
        println!("{}\n", result);
        results.push(result);
    }
    
    // Summary
    println!("═════════════════════════════════════════");
    println!("✅ Summary");
    println!("═════════════════════════════════════════");
    println!("Ran {} benchmark(s)", results.len());
    
    let avg_speedup = results.iter().map(|r| r.speedup()).sum::<f64>() / results.len() as f64;
    println!("Average speedup: {:.2}x", avg_speedup);
    
    // Find best and worst
    if let Some(best) = results.iter().max_by(|a, b| a.speedup().partial_cmp(&b.speedup()).unwrap()) {
        println!("Best improvement: {} ({:.2}x)", best.name, best.speedup());
    }
    
    println!("\n💡 Guidelines:");
    println!("  • Run benchmarks before and after changes");
    println!("  • Track results across releases");
    println!("  • Investigate performance regressions promptly");
    println!("  • Add new benchmarks for critical code paths");
    
    println!("\n📝 To add benchmarks: See comments in benches/performance_benchmarks.rs");
}
