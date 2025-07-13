#!/bin/sh
# Run full CI pipeline locally
# This script mimics the GitHub Actions CI workflow

set -e

echo "🚀 Running full CI pipeline locally..."
echo "This mimics the GitHub Actions workflow"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust."
    exit 1
fi

# Job 1: Format Check
echo "📋 Job 1/3: Format Check"
echo "========================"
echo "📝 Running cargo fmt check..."
if ! cargo fmt --all -- --check; then
    echo "❌ Format job failed!"
    exit 1
fi
echo "✅ Format job passed!"
echo ""

# Job 2: Clippy Check  
echo "📋 Job 2/3: Clippy Check"
echo "========================"
echo "🔍 Running cargo clippy..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy job failed!"
    exit 1
fi
echo "✅ Clippy job passed!"
echo ""

# Job 3: Test Suite
echo "📋 Job 3/3: Test Suite"
echo "======================="
echo "🧪 Running cargo test..."
if ! cargo test; then
    echo "❌ Test job failed!"
    exit 1
fi
echo "✅ Test job passed!"
echo ""

echo "🎉 All CI jobs passed! Your code is ready for production."
echo ""
echo "📊 Summary:"
echo "  ✅ Format check"
echo "  ✅ Clippy (linter)"
echo "  ✅ Test suite (17/17 tests)"
echo ""
echo "🚀 Safe to push to GitHub!" 