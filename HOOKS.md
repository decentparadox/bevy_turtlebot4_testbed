# Pre-commit Hooks Setup

This project includes both automated pre-commit hooks and manual scripts to ensure code quality before commits.

## 🚀 Quick Setup

### Option 1: Using pre-commit framework (Recommended)

1. **Install pre-commit**:
   ```bash
   # On macOS
   brew install pre-commit
   
   # On Ubuntu/Debian
   sudo apt install pre-commit
   
   # Using pip
   pip install pre-commit
   ```

2. **Install the hooks**:
   ```bash
   pre-commit install
   ```

3. **Test the setup**:
   ```bash
   pre-commit run --all-files
   ```

### Option 2: Manual Git Hooks

1. **Set up git hooks directory**:
   ```bash
   git config core.hooksPath .githooks
   ```

2. **Make scripts executable**:
   ```bash
   chmod +x .githooks/pre-commit
   chmod +x .githooks/run-ci.sh
   ```

## 📋 What Gets Checked

### Pre-commit Hooks
- **Formatting**: `cargo fmt --all -- --check`
- **Linting**: `cargo clippy -- -D warnings`
- **Compilation**: `cargo check --all-targets --all-features`
- **Basic checks**: trailing whitespace, file endings, YAML syntax

### Full CI Pipeline
Run locally with: `.githooks/run-ci.sh`
- **Format Check**: Ensures code follows Rust style guidelines
- **Clippy Check**: Catches common mistakes and enforces best practices
- **Test Suite**: Runs all 17 sensor validation tests

## 🛠️ Usage

### Automatic (Pre-commit Framework)
Hooks run automatically on `git commit`. If checks fail, the commit is blocked.

### Manual Commands
```bash
# Run pre-commit checks only
.githooks/pre-commit

# Run full CI pipeline locally
.githooks/run-ci.sh

# Run individual checks
cargo fmt --all -- --check    # Format check
cargo clippy -- -D warnings   # Linting
cargo test                     # Test suite
```

### Skip Hooks (Emergency Only)
```bash
# Skip pre-commit hooks (not recommended)
git commit --no-verify -m "Emergency fix"

# Run tests manually later
pre-commit run --all-files
```

## 🔧 Customization

### Enable Tests in Pre-commit
Edit `.githooks/pre-commit` and uncomment the test section:
```bash
# Uncomment these lines to run tests on every commit
echo "🧪 Running tests..."
if ! cargo test; then
    echo "❌ Tests failed!"
    exit 1
fi
```

### Pre-commit Configuration
Edit `.pre-commit-config.yaml` to:
- Add new hooks
- Change which hooks run on commit vs manually
- Adjust hook settings

## 📊 CI Integration

### GitHub Actions
The `.github/workflows/ci.yml` file runs the same checks on:
- Push to `main`/`master` branches
- Pull requests

### Local CI Simulation
Run the exact same pipeline locally:
```bash
.githooks/run-ci.sh
```

## 🐛 Troubleshooting

### Hook Installation Issues
```bash
# Reinstall pre-commit hooks
pre-commit uninstall
pre-commit install

# Update hooks to latest versions
pre-commit autoupdate
```

### Permission Issues (Windows)
```powershell
# Make scripts executable
git update-index --chmod=+x .githooks/pre-commit
git update-index --chmod=+x .githooks/run-ci.sh
```

### Skip Specific Hooks
```bash
# Skip only clippy
SKIP=clippy git commit -m "Skip clippy for this commit"

# Skip multiple hooks
SKIP=clippy,fmt git commit -m "Skip multiple hooks"
```

## 📈 Benefits

- **Catch issues early**: Before they reach CI/CD
- **Consistent code quality**: Enforced formatting and linting
- **Faster feedback**: Local checks are faster than CI
- **Team collaboration**: Everyone follows the same standards
- **Robust testing**: 17 comprehensive sensor validation tests 