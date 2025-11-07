# Integration Test Suite

This directory contains comprehensive integration tests for the Humidor application.

## Test Organization

### Test Files

- **`common/mod.rs`** - Shared test utilities and database setup
- **`auth_tests.rs`** - Authentication and user management tests
- **`cigar_tests.rs`** - Cigar CRUD operations and validation tests
- **`favorites_tests.rs`** - Favorites functionality tests
- **`wish_list_tests.rs`** - Wish list functionality tests
- **`quantity_tests.rs`** - Cigar quantity tracking tests
- **`security_tests.rs`** - Security feature tests (rate limiting, password hashing, JWT, CORS, etc.)
- **`integration_tests.rs`** - End-to-end integration tests for API workflows

## Test Coverage

### Security Tests (`security_tests.rs`)
✅ Rate limiter limits login attempts  
✅ Rate limiter clears on successful login  
✅ Rate limiter expires old attempts  
✅ Rate limiter isolates different IPs  
✅ JSON body size limits (1MB)  
✅ Password never stored in plaintext  
✅ JWT tokens have expiration  
✅ Database connection pooling works  
✅ Email validation logic  
✅ CORS origin validation logic  
✅ User IDs are UUIDs  
✅ Timestamps set on creation  

### Integration Tests (`integration_tests.rs`)
✅ Humidor CRUD operations  
✅ Humidor user isolation  
✅ Cigar quantity tracking  
✅ Cigar-humidor relationships  
✅ Favorite persistence  
✅ Wish list with notes  
✅ Unique constraint enforcement  
✅ Cascade delete behavior  
✅ Admin flag functionality  
✅ Concurrent database operations  

### Authentication Tests (`auth_tests.rs`)
✅ User creation  
✅ Admin user creation  
✅ Password hashing  
✅ Username uniqueness  
✅ User profile retrieval  

### Cigar Tests (`cigar_tests.rs`)
✅ Cigar creation  
✅ Cigar with humidor assignment  
✅ Cigar updates  
✅ Cigar deletion  
✅ Active/inactive state management  

### Favorites Tests (`favorites_tests.rs`)
✅ Add favorites  
✅ Get user favorites  
✅ Remove favorites  
✅ Check favorite status  
✅ Prevent duplicate favorites  

### Wish List Tests (`wish_list_tests.rs`)
✅ Add to wish list  
✅ Get wish list items  
✅ Update wish list notes  
✅ Remove from wish list  
✅ Check wish list status  

## Prerequisites

### 1. Database Setup

Tests require a PostgreSQL database. The easiest way is to use Docker Compose:

```bash
# Start PostgreSQL container
docker-compose up -d postgres

# Or use the full stack
docker-compose up -d
```

### 2. Environment Variables

Create a `.env` file or set environment variables:

```bash
# Required for tests
TEST_DATABASE_URL=postgresql://humidor_user:humidor_pass@localhost:5432/humidor_db
JWT_SECRET=test_secret_key_for_testing
```

Or use the default values (tests will fall back to sensible defaults).

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Specific Test Suite

```bash
# Security tests only
cargo test --test security_tests

# Integration tests only
cargo test --test integration_tests

# Authentication tests only
cargo test --test auth_tests

# Cigar tests only
cargo test --test cigar_tests
```

### Run Specific Test

```bash
cargo test test_rate_limiter_limits_login_attempts
```

### Run With Output

```bash
# Show println! output
cargo test -- --nocapture

# Show test names as they run
cargo test -- --test-threads=1 --nocapture
```

### Run Tests in Release Mode

```bash
cargo test --release
```

## Test Features

### Serial Execution

Tests use the `#[serial]` attribute from `serial_test` crate to prevent database conflicts. This ensures tests run one at a time when accessing the database.

### Database Cleanup

The `setup_test_db()` function automatically:
1. Runs database migrations
2. Cleans up existing test data
3. Returns a fresh database connection pool

Each test gets a clean database state.

### Unique Test Data

User creation generates unique usernames with UUIDs to prevent conflicts:
```rust
let (user_id, username) = create_test_user(&pool, "testuser", "password", false).await?;
// Username will be like: testuser_550e8400-e29b-41d4-a716-446655440000
```

## Common Test Utilities

### Database Setup
```rust
let ctx = setup_test_db().await;
let pool = ctx.pool;
```

### Create Test User
```rust
let (user_id, username) = create_test_user(
    &pool,
    "username",
    "password",
    false  // is_admin
).await?;
```

### Create Test Humidor
```rust
let humidor_id = create_test_humidor(&pool, user_id, "Humidor Name").await?;
```

### Create Test Cigar
```rust
let cigar_id = create_test_cigar(
    &pool,
    "Cigar Name",
    10,  // quantity
    Some(humidor_id)  // optional humidor
).await?;
```

### Create JWT Token
```rust
let token = create_user_and_login(&pool, "username", "password").await?;
```

## Continuous Integration

Tests are designed to run in CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run tests
  run: |
    docker-compose up -d postgres
    sleep 5  # Wait for postgres to be ready
    cargo test
  env:
    TEST_DATABASE_URL: postgresql://humidor_user:humidor_pass@localhost:5432/humidor_db
    JWT_SECRET: test_secret_key_for_ci
```

## Troubleshooting

### "Failed to connect to database"
- Ensure PostgreSQL is running: `docker-compose ps`
- Check connection string in `TEST_DATABASE_URL`
- Verify database credentials

### "Table does not exist"
- Migrations may not have run
- Check migrations directory exists
- Verify `refinery` is properly configured

### "Tests hanging"
- Database might be locked by another process
- Try running with `--test-threads=1`
- Check for deadlocks in test code

### "Unique constraint violation"
- Test cleanup may have failed
- Restart database: `docker-compose restart postgres`
- Use `#[serial]` attribute for database tests

## Test Performance

- **Full test suite**: ~30-60 seconds
- **Single test file**: ~5-15 seconds
- **Individual test**: <1 second (typically)

Tests use `#[serial]` for database safety which slows execution but ensures reliability.

## Adding New Tests

1. Follow existing test patterns
2. Use `#[serial]` for database tests
3. Use `setup_test_db()` for database access
4. Generate unique test data (UUIDs)
5. Clean up test data (automatic with setup)
6. Add descriptive test names
7. Document what the test validates

Example:
```rust
#[tokio::test]
#[serial]
async fn test_my_new_feature() {
    let ctx = setup_test_db().await;
    
    // Setup test data
    let (user_id, _) = create_test_user(&ctx.pool, "testuser", "pass", false).await.unwrap();
    
    // Test your feature
    // ...
    
    // Assert expected behavior
    assert!(/* condition */);
}
```

## Code Coverage

To generate code coverage reports:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# Open coverage/index.html in browser
```

## Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Use `setup_test_db()` for automatic cleanup
3. **Assertions**: Use descriptive assertion messages
4. **Naming**: Use `test_feature_behavior` naming convention
5. **Documentation**: Add comments for complex test logic
6. **Serial**: Use `#[serial]` for database tests
7. **Async**: Always use `#[tokio::test]` for async tests

## Test Maintenance

- Keep tests up-to-date with code changes
- Run tests before committing changes
- Fix failing tests immediately
- Add tests for new features
- Update test documentation
- Review test coverage regularly
