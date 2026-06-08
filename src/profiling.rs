use std::time::{Duration, Instant};
use std::path::Path;

/// Performance profiling utilities for VHS-86
pub struct Profiler {
    name: String,
    start: Instant,
}

impl Profiler {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn report(&self) -> String {
        format!("[{}] elapsed: {:?}", self.name, self.elapsed())
    }
}

impl Drop for Profiler {
    fn drop(&mut self) {
        eprintln!("{}", self.report());
    }
}

/// Profile a directory listing operation
pub fn profile_directory_listing(path: &Path) -> (Duration, usize) {
    let start = Instant::now();
    let count = std::fs::read_dir(path)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);
    let elapsed = start.elapsed();
    (elapsed, count)
}

/// Profile a file read operation
pub fn profile_file_read(path: &Path) -> (Duration, usize) {
    let start = Instant::now();
    let size = std::fs::read(path).map(|data| data.len()).unwrap_or(0);
    let elapsed = start.elapsed();
    (elapsed, size)
}

/// Benchmark result with statistics
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

impl BenchmarkResult {
    pub fn throughput_per_sec(&self, items_per_iteration: usize) -> f64 {
        let secs = self.total_duration.as_secs_f64();
        if secs > 0.0 {
            (self.iterations * items_per_iteration) as f64 / secs
        } else {
            0.0
        }
    }

    pub fn report(&self) -> String {
        format!(
            "Benchmark: {}\n  iterations: {}\n  total: {:?}\n  avg: {:?}\n  min: {:?}\n  max: {:?}\n  throughput: {:.0} ops/sec",
            self.name,
            self.iterations,
            self.total_duration,
            self.avg_duration,
            self.min_duration,
            self.max_duration,
            self.throughput_per_sec(1)
        )
    }
}

/// Run a benchmark function multiple times and collect statistics
pub fn benchmark<F>(name: &str, iterations: usize, mut f: F) -> BenchmarkResult
where
    F: FnMut(),
{
    let mut durations = Vec::with_capacity(iterations);
    let total_start = Instant::now();

    for _ in 0..iterations {
        let start = Instant::now();
        f();
        durations.push(start.elapsed());
    }

    let total_duration = total_start.elapsed();

    durations.sort();
    let min_duration = *durations.first().unwrap_or(&Duration::ZERO);
    let max_duration = *durations.last().unwrap_or(&Duration::ZERO);
    let avg_duration = if !durations.is_empty() {
        total_duration / iterations as u32
    } else {
        Duration::ZERO
    };

    BenchmarkResult {
        name: name.to_string(),
        iterations,
        total_duration,
        avg_duration,
        min_duration,
        max_duration,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = Profiler::new("test");
        assert_eq!(profiler.name, "test");
    }

    #[test]
    fn test_profiler_elapsed() {
        let profiler = Profiler::new("test");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = profiler.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_profiler_report() {
        let profiler = Profiler::new("test");
        let report = profiler.report();
        assert!(report.contains("test"));
        assert!(report.contains("elapsed"));
    }

    #[test]
    fn test_benchmark_result_report() {
        let result = BenchmarkResult {
            name: "test".to_string(),
            iterations: 10,
            total_duration: Duration::from_millis(100),
            avg_duration: Duration::from_millis(10),
            min_duration: Duration::from_millis(5),
            max_duration: Duration::from_millis(20),
        };
        let report = result.report();
        assert!(report.contains("test"));
        assert!(report.contains("iterations: 10"));
        assert!(report.contains("throughput"));
    }

    #[test]
    fn test_benchmark_result_throughput() {
        let result = BenchmarkResult {
            name: "test".to_string(),
            iterations: 100,
            total_duration: Duration::from_secs(1),
            avg_duration: Duration::from_millis(10),
            min_duration: Duration::from_millis(5),
            max_duration: Duration::from_millis(20),
        };
        assert_eq!(result.throughput_per_sec(1), 100.0);
    }

    #[test]
    fn test_benchmark_execution() {
        let result = benchmark("noop", 10, || {});
        assert_eq!(result.iterations, 10);
        assert_eq!(result.name, "noop");
    }

    #[test]
    fn test_profile_directory_listing() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::write(temp_dir.path().join("file.txt"), "test").unwrap();
        
        let (duration, count) = profile_directory_listing(temp_dir.path());
        assert!(duration.as_nanos() > 0);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_profile_file_read() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        std::fs::write(&file_path, "hello world").unwrap();
        
        let (duration, size) = profile_file_read(&file_path);
        assert!(duration.as_nanos() > 0);
        assert_eq!(size, 11);
    }
}
