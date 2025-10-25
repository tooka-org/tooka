# Performance Benchmarks

This directory contains the performance benchmark suite for Tooka. The benchmarks help track performance across releases, identify regressions, and validate optimization improvements.

## Running Benchmarks

To run the complete benchmark suite:

```bash
cargo run --release --bin performance_benchmarks
```

Or build and run directly:

```bash
cargo build --release --bin performance_benchmarks
./target/release/performance_benchmarks
```

## Purpose

The benchmark suite serves several key purposes:

1. **Track Performance**: Monitor performance trends across releases and commits
2. **Catch Regressions**: Identify performance degradations early in development
3. **Validate Optimizations**: Confirm that optimizations deliver real improvements
4. **Guide Development**: Highlight areas that would benefit from optimization

## Current Benchmarks

### 1. Regex Compilation Caching
- **What**: Template pattern matching with compiled regex
- **Optimization**: Regex compiled once with `LazyLock` instead of on every call
- **Expected Impact**: 100x+ speedup in template processing
- **Use Case**: File renaming with template patterns

### 2. Date Constant Caching
- **What**: Date constant creation for range comparisons
- **Optimization**: Date constants cached with `LazyLock` instead of recreated
- **Expected Impact**: Eliminates repeated allocations
- **Use Case**: Filtering files by creation/modification dates

### 3. Extension Matching
- **What**: String comparison in file extension checking
- **Optimization**: Direct `as_str()` comparison instead of `String` equality
- **Expected Impact**: 10-20% speedup in hot path
- **Use Case**: File filtering by extension

## Adding New Benchmarks

To add a new benchmark to the suite:

### 1. Create a Benchmark Struct

```rust
struct MyNewBenchmark;

impl Benchmark for MyNewBenchmark {
    fn name(&self) -> &str {
        "My New Benchmark"
    }
    
    fn description(&self) -> &str {
        "What this benchmark measures"
    }
    
    fn run(&self) -> BenchmarkResult {
        // Baseline implementation
        let baseline_start = Instant::now();
        for _ in 0..iterations {
            // Old approach
        }
        let baseline_duration = baseline_start.elapsed();
        
        // Optimized implementation
        let optimized_start = Instant::now();
        for _ in 0..iterations {
            // New approach
        }
        let optimized_duration = optimized_start.elapsed();
        
        BenchmarkResult {
            name: self.name().to_string(),
            description: self.description().to_string(),
            baseline_duration,
            optimized_duration,
        }
    }
}
```

### 2. Register the Benchmark

Add your benchmark to the `benchmarks` vector in `main()`:

```rust
let benchmarks: Vec<Box<dyn Benchmark>> = vec![
    // ... existing benchmarks
    Box::new(MyNewBenchmark),
];
```

### 3. Document Expected Behavior

Update this README with:
- What the benchmark measures
- The optimization being validated
- Expected performance characteristics
- Real-world use cases

## Best Practices

### When to Run Benchmarks

- **Before major releases**: Ensure no performance regressions
- **After optimization work**: Validate improvements are real
- **During code review**: Compare performance impact of changes
- **Periodically**: Track trends over time

### Interpreting Results

- **Speedup > 1.0**: Optimization is faster (good!)
- **Speedup < 1.0**: Optimization is slower (investigate)
- **High variance**: Test may need more iterations or better isolation

### Adding Quality Benchmarks

Good benchmarks should:
- Test realistic scenarios from actual usage
- Use `std::hint::black_box` to prevent optimization
- Run enough iterations for stable measurements
- Compare baseline vs optimized implementations
- Document expected performance characteristics

## Performance Tracking

Consider tracking benchmark results over time:

```bash
# Run and save results with timestamp
cargo run --release --bin performance_benchmarks > "benchmark_$(date +%Y%m%d).txt"
```

This creates a historical record for trend analysis.

## CI Integration

To integrate benchmarks into CI:

```yaml
- name: Run performance benchmarks
  run: cargo run --release --bin performance_benchmarks
```

Add thresholds or comparison logic to fail CI if performance regresses significantly.

## Notes

- Benchmarks run in release mode with full optimizations
- Results may vary based on system load and hardware
- Focus on relative performance (speedup) rather than absolute times
- Use consistent hardware for meaningful comparisons across commits
