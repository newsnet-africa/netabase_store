#!/bin/bash

# Comprehensive test runner for NetabaseStore
# This script runs all categories of tests with proper coordination via nextest

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-target}
TEST_THREADS=${TEST_THREADS:-$(nproc)}
BENCH_TIME=${BENCH_TIME:-10}
COVERAGE_DIR="target/coverage"
REPORTS_DIR="target/test-reports"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Setup function to install required tools
setup_tools() {
    print_status "Setting up test tools..."
    
    # Check if nextest is installed
    if ! command_exists cargo-nextest; then
        print_status "Installing cargo-nextest..."
        cargo install cargo-nextest
    fi
    
    # Check if criterion is available for benchmarks
    if ! command_exists cargo-criterion; then
        print_status "Installing cargo-criterion..."
        cargo install cargo-criterion
    fi
    
    # Check if coverage tools are available
    if ! command_exists cargo-llvm-cov; then
        print_status "Installing cargo-llvm-cov..."
        cargo install cargo-llvm-cov
    fi
    
    # Check if audit tool is available
    if ! command_exists cargo-audit; then
        print_status "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    print_success "Tools setup complete"
}

# Function to create directories
create_directories() {
    print_status "Creating output directories..."
    mkdir -p "$REPORTS_DIR"
    mkdir -p "$COVERAGE_DIR"
    mkdir -p "$CARGO_TARGET_DIR/nextest"
}

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    
    cargo nextest run \
        --profile default \
        --test-threads="$TEST_THREADS" \
        --filter-expr 'test(/^unit_/)' \
        --no-capture \
        2>&1 | tee "$REPORTS_DIR/unit_tests.log"
    
    local exit_code=${PIPESTATUS[0]}
    
    if [ $exit_code -eq 0 ]; then
        print_success "Unit tests passed"
    else
        print_error "Unit tests failed with exit code $exit_code"
        return $exit_code
    fi
}

# Function to run component tests
run_component_tests() {
    print_status "Running component tests..."
    
    cargo nextest run \
        --profile default \
        --test-threads="$TEST_THREADS" \
        --filter-expr 'test(/^component_/)' \
        --no-capture \
        2>&1 | tee "$REPORTS_DIR/component_tests.log"
    
    local exit_code=${PIPESTATUS[0]}
    
    if [ $exit_code -eq 0 ]; then
        print_success "Component tests passed"
    else
        print_error "Component tests failed with exit code $exit_code"
        return $exit_code
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_status "Running integration tests..."
    
    cargo nextest run \
        --profile default \
        --test-threads="$TEST_THREADS" \
        --filter-expr 'test(/^integration_/)' \
        --no-capture \
        2>&1 | tee "$REPORTS_DIR/integration_tests.log"
    
    local exit_code=${PIPESTATUS[0]}
    
    if [ $exit_code -eq 0 ]; then
        print_success "Integration tests passed"
    else
        print_error "Integration tests failed with exit code $exit_code"
        return $exit_code
    fi
}

# Function to run API tests
run_api_tests() {
    print_status "Running API tests..."
    
    cargo nextest run \
        --profile default \
        --test-threads="$TEST_THREADS" \
        --filter-expr 'test(/^api_/)' \
        --no-capture \
        2>&1 | tee "$REPORTS_DIR/api_tests.log"
    
    local exit_code=${PIPESTATUS[0]}
    
    if [ $exit_code -eq 0 ]; then
        print_success "API tests passed"
    else
        print_error "API tests failed with exit code $exit_code"
        return $exit_code
    fi
}

# Function to run all tests in sequence
run_all_tests() {
    print_status "Running all tests in sequence..."
    
    run_unit_tests
    run_component_tests
    run_integration_tests
    run_api_tests
    
    print_success "All test categories completed"
}

# Function to run benchmarks
run_benchmarks() {
    print_status "Running performance benchmarks..."
    
    print_status "Running comprehensive benchmarks..."
    cargo bench --bench bench_comprehensive -- --output-format html 2>&1 | tee "$REPORTS_DIR/bench_comprehensive.log"
    
    print_status "Running memory benchmarks..."
    cargo bench --bench bench_memory -- --output-format html 2>&1 | tee "$REPORTS_DIR/bench_memory.log"
    
    print_status "Running concurrency benchmarks..."
    cargo bench --bench bench_concurrency -- --output-format html 2>&1 | tee "$REPORTS_DIR/bench_concurrency.log"
    
    print_status "Running throughput benchmarks..."
    cargo bench --bench bench_throughput 2>&1 | tee "$REPORTS_DIR/bench_throughput.log"
    
    print_success "Benchmarks completed"
}

# Function to generate test coverage
generate_coverage() {
    print_status "Generating test coverage report..."
    
    cargo llvm-cov nextest \
        --html \
        --output-dir "$COVERAGE_DIR" \
        --ignore-filename-regex "/(tests|benches|examples)/" \
        2>&1 | tee "$REPORTS_DIR/coverage.log"
    
    local exit_code=${PIPESTATUS[0]}
    
    if [ $exit_code -eq 0 ]; then
        print_success "Coverage report generated in $COVERAGE_DIR"
    else
        print_warning "Coverage generation had issues (exit code $exit_code)"
    fi
}

# Function to run security audit
run_security_audit() {
    print_status "Running security audit..."
    
    cargo audit 2>&1 | tee "$REPORTS_DIR/security_audit.log"
    
    local exit_code=${PIPESTATUS[0]}
    
    if [ $exit_code -eq 0 ]; then
        print_success "Security audit passed"
    else
        print_warning "Security audit found issues (exit code $exit_code)"
    fi
}

# Function to run code quality checks
run_quality_checks() {
    print_status "Running code quality checks..."
    
    # Check formatting
    print_status "Checking code formatting..."
    cargo fmt --all -- --check 2>&1 | tee "$REPORTS_DIR/fmt_check.log"
    local fmt_result=$?
    
    # Run clippy
    print_status "Running clippy..."
    cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee "$REPORTS_DIR/clippy.log"
    local clippy_result=$?
    
    # Check for unused dependencies
    if command_exists cargo-udeps; then
        print_status "Checking for unused dependencies..."
        cargo +nightly udeps 2>&1 | tee "$REPORTS_DIR/unused_deps.log"
        local udeps_result=$?
    else
        print_warning "cargo-udeps not installed, skipping unused dependency check"
        local udeps_result=0
    fi
    
    if [ $fmt_result -eq 0 ] && [ $clippy_result -eq 0 ] && [ $udeps_result -eq 0 ]; then
        print_success "Code quality checks passed"
        return 0
    else
        print_error "Code quality checks failed"
        return 1
    fi
}

# Function to generate comprehensive test report
generate_test_report() {
    print_status "Generating comprehensive test report..."
    
    local report_file="$REPORTS_DIR/test_summary.md"
    
    cat > "$report_file" << EOF
# NetabaseStore Test Report

Generated on: $(date)
Test threads: $TEST_THREADS
Rust version: $(rustc --version)
Cargo version: $(cargo --version)

## Test Categories

### Unit Tests
- **Purpose**: Test individual functions and small components in isolation
- **Location**: \`tests/unit_tests.rs\`
- **Test Count**: $(grep -c "#\[test\]" tests/unit_tests.rs || echo "Unknown")
- **Status**: $(if [ -f "$REPORTS_DIR/unit_tests.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)

### Component Tests  
- **Purpose**: Test database components with full functionality
- **Location**: \`tests/component_tests.rs\`
- **Test Count**: $(grep -c "#\[test\]" tests/component_tests.rs || echo "Unknown")
- **Status**: $(if [ -f "$REPORTS_DIR/component_tests.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)

### Integration Tests
- **Purpose**: Test communication between traits and implementations
- **Location**: \`tests/integration_tests.rs\`
- **Test Count**: $(grep -c "#\[test\]" tests/integration_tests.rs || echo "Unknown")
- **Status**: $(if [ -f "$REPORTS_DIR/integration_tests.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)

### API Tests
- **Purpose**: Test comprehensive mock application using the full API
- **Location**: \`tests/api_tests.rs\`
- **Test Count**: $(grep -c "#\[test\]" tests/api_tests.rs || echo "Unknown")
- **Status**: $(if [ -f "$REPORTS_DIR/api_tests.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)

## Benchmarks

### Performance Benchmarks
- **Comprehensive**: $(if [ -f "$REPORTS_DIR/bench_comprehensive.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)
- **Memory**: $(if [ -f "$REPORTS_DIR/bench_memory.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)
- **Concurrency**: $(if [ -f "$REPORTS_DIR/bench_concurrency.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)
- **Throughput**: $(if [ -f "$REPORTS_DIR/bench_throughput.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)

## Code Quality

### Coverage
- **Report**: $(if [ -d "$COVERAGE_DIR" ]; then echo "📊 Available in $COVERAGE_DIR"; else echo "❌ Not generated"; fi)

### Security
- **Audit**: $(if [ -f "$REPORTS_DIR/security_audit.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)

### Formatting & Linting
- **Format Check**: $(if [ -f "$REPORTS_DIR/fmt_check.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)
- **Clippy**: $(if [ -f "$REPORTS_DIR/clippy.log" ]; then echo "✅ Completed"; else echo "❌ Not run"; fi)

## Files Generated

EOF

    if [ -d "$REPORTS_DIR" ]; then
        find "$REPORTS_DIR" -type f -name "*.log" | while read -r file; do
            echo "- \`$(basename "$file")\` ($(wc -l < "$file") lines)" >> "$report_file"
        done
    fi
    
    if [ -d "$COVERAGE_DIR" ]; then
        echo "" >> "$report_file"
        echo "### Coverage Files" >> "$report_file"
        find "$COVERAGE_DIR" -type f | while read -r file; do
            echo "- \`$(basename "$file")\`" >> "$report_file"
        done
    fi
    
    print_success "Test report generated: $report_file"
}

# Function to clean up temporary files
cleanup() {
    print_status "Cleaning up temporary files..."
    
    # Remove any leftover test databases
    find . -name "*.db" -path "./target/*" -delete 2>/dev/null || true
    find . -name "test_*" -path "./target/*" -type d -exec rm -rf {} + 2>/dev/null || true
    
    print_success "Cleanup completed"
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 [COMMAND]

Commands:
    setup           Install required tools
    unit            Run unit tests only
    component       Run component tests only  
    integration     Run integration tests only
    api             Run API tests only
    all-tests       Run all test categories
    bench           Run all benchmarks
    coverage        Generate test coverage report
    audit           Run security audit
    quality         Run code quality checks
    report          Generate comprehensive test report
    full            Run everything (tests, benchmarks, coverage, quality)
    clean           Clean up temporary files
    help            Show this help message

Environment Variables:
    TEST_THREADS    Number of test threads (default: number of CPUs)
    BENCH_TIME      Benchmark time in seconds (default: 10)
    CARGO_TARGET_DIR Target directory for builds (default: target)

Examples:
    $0 setup                    # Install required tools
    $0 all-tests                # Run all tests
    $0 full                     # Run everything
    TEST_THREADS=1 $0 unit      # Run unit tests with 1 thread
    $0 bench                    # Run benchmarks only

EOF
}

# Main function
main() {
    local command=${1:-help}
    
    case "$command" in
        setup)
            setup_tools
            ;;
        unit)
            create_directories
            run_unit_tests
            ;;
        component)
            create_directories
            run_component_tests
            ;;
        integration) 
            create_directories
            run_integration_tests
            ;;
        api)
            create_directories
            run_api_tests
            ;;
        all-tests)
            create_directories
            run_all_tests
            ;;
        bench)
            create_directories
            run_benchmarks
            ;;
        coverage)
            create_directories
            generate_coverage
            ;;
        audit)
            create_directories
            run_security_audit
            ;;
        quality)
            create_directories
            run_quality_checks
            ;;
        report)
            create_directories
            generate_test_report
            ;;
        full)
            print_status "Running comprehensive test suite..."
            create_directories
            run_quality_checks
            run_all_tests
            generate_coverage
            run_benchmarks
            run_security_audit
            generate_test_report
            cleanup
            print_success "Full test suite completed!"
            ;;
        clean)
            cleanup
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            print_error "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Run main function with all arguments
main "$@"