//! Test utilities and helper functions for NetabaseStore testing
//! 
//! This module provides common utilities, fixtures, and helper functions
//! used across all test categories.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use serde::{Deserialize, Serialize};
use rand::{Rng, SeedableRng, rngs::StdRng};

/// Global test configuration
static TEST_CONFIG_LAZY: std::sync::LazyLock<TestConfig> = std::sync::LazyLock::new(|| {
    TestConfig::default()
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub seed: u64,
    pub temp_dir: String,
    pub verbose: bool,
    pub timeout: Duration,
    pub max_db_size: u64,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            temp_dir: "target/test-data".to_string(),
            verbose: false,
            timeout: Duration::from_secs(300),
            max_db_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// Initialize test configuration (call once at start of test suite)
pub fn init_test_config(_config: Option<TestConfig>) {
    // With LazyLock, initialization is handled automatically
    // This function is kept for compatibility but does nothing
}

/// Get the global test configuration
pub fn get_test_config() -> &'static TestConfig {
    &TEST_CONFIG_LAZY
}

/// Test data generator with consistent seeding
pub struct TestDataGenerator {
    rng: StdRng,
    counter: u64,
}

impl TestDataGenerator {
    pub fn new() -> Self {
        let config = get_test_config();
        Self {
            rng: StdRng::seed_from_u64(config.seed),
            counter: 0,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            counter: 0,
        }
    }

    pub fn next_id(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    pub fn random_string(&mut self, min_len: usize, max_len: usize) -> String {
        let len = self.rng.gen_range(min_len..=max_len);
        (0..len)
            .map(|_| {
                let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
                chars[self.rng.gen_range(0..chars.len())] as char
            })
            .collect()
    }

    pub fn random_email(&mut self) -> String {
        let username = self.random_string(5, 15);
        let domain = self.random_string(5, 10);
        let tld = ["com", "org", "net", "edu"][self.rng.gen_range(0..4)];
        format!("{}@{}.{}", username, domain, tld)
    }

    pub fn random_bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.rng.r#gen()).collect()
    }

    pub fn random_timestamp(&mut self) -> u64 {
        let base = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        base - self.rng.gen_range(0..31536000) // Up to 1 year ago
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Temporary database manager for tests
pub struct TestDatabaseManager {
    temp_dirs: Arc<Mutex<Vec<TempDir>>>,
}

impl TestDatabaseManager {
    pub fn new() -> Self {
        Self {
            temp_dirs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn create_temp_db(&self) -> (TempDir, String) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let _db_path = temp_dir.path().join("test.db").to_string_lossy().to_string();
        
        {
            let mut dirs = self.temp_dirs.lock().unwrap();
            dirs.push(temp_dir);
        }
        
        let dirs = self.temp_dirs.lock().unwrap();
        let temp_dir_ref = dirs.last().unwrap();
        let _db_path = temp_dir_ref.path().join("test.db").to_string_lossy().to_string();
        
        // We can't return the reference, so create a new TempDir
        let new_temp_dir = TempDir::new().expect("Failed to create temp directory");
        let new_db_path = new_temp_dir.path().join("test.db").to_string_lossy().to_string();
        
        (new_temp_dir, new_db_path)
    }

    pub fn cleanup(&self) {
        let mut dirs = self.temp_dirs.lock().unwrap();
        dirs.clear();
    }
}

impl Drop for TestDatabaseManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Performance measurement utilities
pub struct PerformanceMeter {
    measurements: HashMap<String, Vec<Duration>>,
}

impl PerformanceMeter {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
        }
    }

    pub fn measure<T, F>(&mut self, name: &str, operation: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = std::time::Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        self.measurements
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
            
        result
    }

    pub fn get_stats(&self, name: &str) -> Option<PerformanceStats> {
        self.measurements.get(name).map(|durations| {
            let mut sorted = durations.clone();
            sorted.sort();
            
            let count = sorted.len();
            let total = sorted.iter().sum::<Duration>();
            let mean = total / count as u32;
            
            let median = if count % 2 == 0 {
                (sorted[count / 2 - 1] + sorted[count / 2]) / 2
            } else {
                sorted[count / 2]
            };
            
            let min = sorted[0];
            let max = sorted[count - 1];
            
            let p95_idx = (count as f64 * 0.95) as usize;
            let p95 = sorted[p95_idx.min(count - 1)];
            
            PerformanceStats {
                count,
                total,
                mean,
                median,
                min,
                max,
                p95,
            }
        })
    }

    pub fn report(&self) -> String {
        let mut report = String::from("Performance Report:\n");
        report.push_str("===================\n\n");
        
        for (name, _) in &self.measurements {
            if let Some(stats) = self.get_stats(name) {
                report.push_str(&format!(
                    "{name}:\n\
                     - Count: {count}\n\
                     - Total: {total:?}\n\
                     - Mean: {mean:?}\n\
                     - Median: {median:?}\n\
                     - Min: {min:?}\n\
                     - Max: {max:?}\n\
                     - P95: {p95:?}\n\n",
                    name = name,
                    count = stats.count,
                    total = stats.total,
                    mean = stats.mean,
                    median = stats.median,
                    min = stats.min,
                    max = stats.max,
                    p95 = stats.p95
                ));
            }
        }
        
        report
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub count: usize,
    pub total: Duration,
    pub mean: Duration,
    pub median: Duration,
    pub min: Duration,
    pub max: Duration,
    pub p95: Duration,
}

/// Test assertion utilities
pub struct TestAssertions;

impl TestAssertions {
    pub fn assert_performance(
        actual: Duration,
        expected_max: Duration,
        operation: &str,
    ) {
        if actual > expected_max {
            panic!(
                "Performance assertion failed for {}: actual={:?}, expected_max={:?}",
                operation, actual, expected_max
            );
        }
    }

    pub fn assert_memory_usage(
        actual_bytes: u64,
        max_bytes: u64,
        operation: &str,
    ) {
        if actual_bytes > max_bytes {
            panic!(
                "Memory assertion failed for {}: actual={}MB, max={}MB",
                operation,
                actual_bytes / (1024 * 1024),
                max_bytes / (1024 * 1024)
            );
        }
    }

    pub fn assert_eventually<F>(
        condition: F,
        timeout: Duration,
        check_interval: Duration,
        description: &str,
    ) where
        F: Fn() -> bool,
    {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            if condition() {
                return;
            }
            std::thread::sleep(check_interval);
        }
        
        panic!("Assertion failed within timeout: {}", description);
    }
}

/// Test data fixtures
pub struct TestFixtures;

#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub age: u32,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct TestOrganization {
    pub id: u64,
    pub name: String,
    pub slug: String,
    pub owner_id: u64,
    pub member_count: u32,
}

#[derive(Debug, Clone)]
pub struct TestProject {
    pub id: u64,
    pub name: String,
    pub organization_id: u64,
    pub owner_id: u64,
    pub status: String,
}

impl TestFixtures {
    pub fn sample_user_data() -> Vec<TestUser> {
        vec![
            TestUser {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                age: 30,
                active: true,
            },
            TestUser {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                age: 25,
                active: true,
            },
            TestUser {
                id: 3,
                username: "charlie".to_string(),
                email: "charlie@example.com".to_string(),
                age: 35,
                active: false,
            },
        ]
    }

    pub fn sample_organization_data() -> Vec<TestOrganization> {
        vec![
            TestOrganization {
                id: 1,
                name: "TechCorp".to_string(),
                slug: "techcorp".to_string(),
                owner_id: 1,
                member_count: 50,
            },
            TestOrganization {
                id: 2,
                name: "DataSystems".to_string(),
                slug: "datasystems".to_string(),
                owner_id: 2,
                member_count: 25,
            },
        ]
    }

    pub fn sample_project_data() -> Vec<TestProject> {
        vec![
            TestProject {
                id: 1,
                name: "Web Platform".to_string(),
                organization_id: 1,
                owner_id: 1,
                status: "active".to_string(),
            },
            TestProject {
                id: 2,
                name: "Mobile App".to_string(),
                organization_id: 1,
                owner_id: 2,
                status: "active".to_string(),
            },
            TestProject {
                id: 3,
                name: "Analytics Dashboard".to_string(),
                organization_id: 2,
                owner_id: 2,
                status: "archived".to_string(),
            },
        ]
    }
}

/// Concurrent test utilities
pub struct ConcurrentTestUtils;

impl ConcurrentTestUtils {
    pub fn run_concurrent_operations<F>(
        thread_count: usize,
        operations_per_thread: usize,
        operation: F,
    ) -> Vec<std::thread::JoinHandle<()>>
    where
        F: Fn(usize, usize) + Send + Sync + Clone + 'static,
    {
        (0..thread_count)
            .map(|thread_id| {
                let op = operation.clone();
                std::thread::spawn(move || {
                    for op_id in 0..operations_per_thread {
                        op(thread_id, op_id);
                    }
                })
            })
            .collect()
    }

    pub fn wait_for_completion(handles: Vec<std::thread::JoinHandle<()>>) {
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    pub fn stress_test<F>(
        duration: Duration,
        thread_count: usize,
        operation: F,
    ) -> StressTestResults
    where
        F: Fn() + Send + Sync + Clone + 'static + std::panic::RefUnwindSafe,
    {
        let start = std::time::Instant::now();
        let operations_completed = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let errors_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
        
        let handles: Vec<_> = (0..thread_count)
            .map(|_| {
                let op = operation.clone();
                let ops_counter = Arc::clone(&operations_completed);
                let err_counter = Arc::clone(&errors_count);
                
                std::thread::spawn(move || {
                    while start.elapsed() < duration {
                        match std::panic::catch_unwind(|| op()) {
                            Ok(_) => {
                                ops_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            Err(_) => {
                                err_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    }
                })
            })
            .collect();
            
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        StressTestResults {
            duration: start.elapsed(),
            operations_completed: operations_completed.load(std::sync::atomic::Ordering::Relaxed),
            errors_count: errors_count.load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StressTestResults {
    pub duration: Duration,
    pub operations_completed: u64,
    pub errors_count: u64,
}

impl StressTestResults {
    pub fn operations_per_second(&self) -> f64 {
        self.operations_completed as f64 / self.duration.as_secs_f64()
    }

    pub fn error_rate(&self) -> f64 {
        if self.operations_completed + self.errors_count == 0 {
            0.0
        } else {
            self.errors_count as f64 / (self.operations_completed + self.errors_count) as f64
        }
    }
}

/// Macro for creating test data structures
#[macro_export]
macro_rules! test_data {
    ($struct_name:ident {
        $($field:ident: $value:expr),*
    }) => {
        $struct_name {
            $($field: $value),*
        }
    };
}

/// Macro for performance testing
#[macro_export]
macro_rules! assert_performance {
    ($operation:expr, $max_duration:expr) => {
        let start = std::time::Instant::now();
        let _result = $operation;
        let duration = start.elapsed();
        assert!(
            duration <= $max_duration,
            "Operation took too long: {:?} > {:?}",
            duration,
            $max_duration
        );
    };
}

/// Macro for creating parameterized tests
#[macro_export]
macro_rules! parameterized_test {
    ($test_name:ident, $param_type:ty, [$($param:expr),+], $test_body:expr) => {
        #[cfg(test)]
        mod $test_name {
            use super::*;
            
            $(
                #[test]
                fn test() {
                    let param: $param_type = $param;
                    $test_body(param);
                }
            )+
        }
    };
}