# CI/CD Pipeline Documentation

This directory contains GitHub Actions workflows for continuous integration, deployment, and maintenance.

## Workflows

### 1. CI Pipeline (`ci.yml`)

**Triggers**: Push to main/develop/review branches, Pull Requests

**Jobs**:
- ✅ **Lint** - Code formatting (rustfmt) and linting (clippy)
- ✅ **Security** - Security audit for dependencies (cargo-audit)
- ✅ **Test** - Build and run all tests with PostgreSQL
- ✅ **Docker** - Build and validate Docker image
- ✅ **Coverage** - Generate code coverage reports (PRs only)
- ✅ **Dependencies** - Check for outdated and unused dependencies

**Caching**: Cargo registry, index, and build artifacts are cached for faster builds.

**Required Secrets**: None (uses public PostgreSQL)

### 2. Deployment Pipeline (`deploy.yml`)

**Triggers**: Push to main, version tags (v*.*.*), Manual dispatch

**Jobs**:
- 🚀 **Build and Push** - Build Docker image and push to GitHub Container Registry
- 🔒 **Security Scan** - Scan image for vulnerabilities with Trivy
- 📦 **Deploy** - Deploy to production/staging environment

**Required Secrets**:
- `GITHUB_TOKEN` (automatically provided)
- Add deployment-specific secrets as needed (SSH keys, cloud credentials, etc.)

**Environments**:
- `production` - Main production environment
- `staging` - Staging/testing environment (manual trigger)

### 3. Scheduled Maintenance (`scheduled.yml`)

**Triggers**: Weekly (Mondays at 9 AM UTC), Manual dispatch

**Jobs**:
- 📋 **Dependency Updates** - Check for **major version** updates only and create issues
- 🔐 **Security Audit** - Weekly security scan and vulnerability reporting
- 🐳 **Docker Updates** - Check for newer Rust base image versions
- 📊 **Health Summary** - Overall project health status

**Automated Actions**:
- Creates GitHub issues **only for major version** dependency updates (e.g., 1.x → 2.x)
- Minor/patch updates (e.g., 1.2.3 → 1.2.4) are logged but don't create issues
- Creates critical issues for security vulnerabilities
- Notifies about available Rust updates

**Rationale**: Major updates often include breaking changes requiring code changes, while minor/patch updates can be applied with `cargo update` without intervention.

### 4. Pull Request Checks (`pr-checks.yml`)

**Triggers**: PR opened, synchronized, reopened, ready for review

**Jobs**:
- ✅ **Validate** - PR title format, file size checks, TODO detection
- 📊 **Size Check** - Binary size analysis with cargo-bloat
- 🔍 **Breaking Changes** - API compatibility checking
- 📝 **Spell Check** - Spell checking with typos
- 🏷️ **Auto Label** - Automatic PR labeling based on files changed

**Features**:
- Enforces semantic PR titles (feat:, fix:, docs:, etc.)
- Detects large file additions
- Comments on PRs with TODO/FIXME findings
- Provides binary size analysis
- Auto-labels PRs by affected areas

## Configuration Files

### `.github/labeler.yml`
Defines rules for automatic PR labeling based on changed files.

### `.typos.toml`
Configuration for spell checking, includes project-specific dictionary.

## Running Workflows Locally

### Lint and Format
```bash
# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

### Security Audit
```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit
```

### Tests
```bash
# Start PostgreSQL
docker-compose up -d postgres

# Run tests
TEST_DATABASE_URL=postgresql://humidor_user:humidor_pass@localhost:5432/humidor_db \
JWT_SECRET=test_secret cargo test
```

### Docker Build
```bash
# Build image
docker build -t humidor:local .

# Test with docker-compose
docker-compose build
docker-compose up
```

### Code Coverage
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage
```

## Setting Up New Environments

### GitHub Environments

1. Go to **Settings** → **Environments**
2. Create environments: `production`, `staging`
3. Configure protection rules:
   - Required reviewers
   - Wait timer
   - Deployment branches

### Secrets Configuration

Add these secrets in **Settings** → **Secrets and variables** → **Actions**:

**For Deployment**:
- `DEPLOY_SSH_KEY` - SSH key for server access
- `DEPLOY_HOST` - Deployment server hostname
- `DEPLOY_USER` - Deployment user
- Any cloud provider credentials (AWS, GCP, Azure)

**For Notifications** (optional):
- `SLACK_WEBHOOK` - Slack webhook for notifications
- `DISCORD_WEBHOOK` - Discord webhook
- `TELEGRAM_BOT_TOKEN` - Telegram bot token

## Branch Protection Rules

**Note**: Branch protection is **optional** and not required initially. Run the workflows for a week to ensure they work smoothly before enforcing them.

Recommended branch protection for `main` (when ready to enable):

1. ✅ Require pull request before merging
2. ✅ Require approvals (1-2 reviewers)
3. ✅ Require status checks to pass:
   - `Lint and Format Check`
   - `Build and Test`
   - `Security Audit`
   - `Docker Build`
4. ✅ Require branches to be up to date
5. ✅ Require linear history
6. ✅ Include administrators

**Getting Started without Branch Protection:**
1. Push workflows to GitHub
2. Monitor Actions tab for first few runs
3. Fix any issues that arise
4. After 1-2 weeks of stable builds, consider enabling branch protection

## Workflow Badges

Add these badges to your README.md:

```markdown
[![CI](https://github.com/VictoryTek/humidor/actions/workflows/ci.yml/badge.svg)](https://github.com/VictoryTek/humidor/actions/workflows/ci.yml)
[![Deploy](https://github.com/VictoryTek/humidor/actions/workflows/deploy.yml/badge.svg)](https://github.com/VictoryTek/humidor/actions/workflows/deploy.yml)
[![Security Audit](https://github.com/VictoryTek/humidor/actions/workflows/scheduled.yml/badge.svg)](https://github.com/VictoryTek/humidor/actions/workflows/scheduled.yml)
```

## Customizing Workflows

### Adjusting Test Timeout
Edit `ci.yml`:
```yaml
- name: Run integration tests
  timeout-minutes: 15  # Adjust as needed
```

### Changing Dependency Check Schedule
Edit `scheduled.yml`:
```yaml
on:
  schedule:
    - cron: '0 9 * * 1'  # Monday at 9 AM UTC
```

### Adding Deployment Steps
Edit `deploy.yml` in the `deploy` job:
```yaml
- name: Deploy to production
  run: |
    # Add your deployment commands
    ssh user@server "docker pull ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest"
    ssh user@server "docker-compose up -d"
```

## Troubleshooting

### Tests Failing in CI
- Check PostgreSQL service is healthy
- Verify environment variables are set
- Check database migration compatibility

### Docker Build Failing
- Verify Dockerfile syntax
- Check for missing dependencies
- Ensure all source files are copied

### Security Scan Alerts
- Review Trivy output in workflow logs
- Update vulnerable dependencies
- Check base image for updates

### Coverage Not Uploading
- Verify Codecov token is set (if using)
- Check coverage file generation
- Review Codecov action logs

## Best Practices

1. **Keep workflows fast**: Use caching, parallel jobs
2. **Fail fast**: Run quick checks (lint, format) before expensive ones (tests)
3. **Secure secrets**: Never log secrets, use GitHub's secret management
4. **Monitor workflows**: Set up notifications for failures
5. **Keep dependencies updated**: Review weekly maintenance reports
6. **Document changes**: Update this README when modifying workflows

## Monitoring and Alerts

### Workflow Status
- View workflow runs: **Actions** tab in GitHub
- Filter by branch, status, or workflow
- Download logs for debugging

### Notifications
Configure in **Settings** → **Notifications**:
- Email on workflow failures
- GitHub mobile app notifications
- Integration with Slack/Discord for team alerts

## Performance Metrics

Typical workflow times:
- **Lint**: ~2-3 minutes
- **Test**: ~5-8 minutes (with caching)
- **Docker Build**: ~3-5 minutes (with cache)
- **Full CI Pipeline**: ~10-15 minutes

## Support and Maintenance

- Review scheduled maintenance issues weekly
- Update GitHub Actions versions quarterly
- Test workflows on feature branches before merging
- Keep this documentation updated with changes
