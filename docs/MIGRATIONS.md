# Database Migration System

This document describes the database migration system implemented using Refinery.

## Overview

The project uses [Refinery](https://github.com/rust-db/refinery) for managing database migrations. Migrations are SQL files stored in the `migrations/` directory and are automatically applied when the application starts.

## Migration Structure

Migrations follow the naming convention: `V{number}__{description}.sql`

Example: `V1__create_users_table.sql`

### Current Migrations

1. **V1__create_users_table.sql** - Creates the users table for authentication
2. **V2__create_humidors_table.sql** - Creates the humidors table
3. **V3__create_organizer_tables.sql** - Creates organizer tables (brands, sizes, origins, strengths, ring_gauges)
4. **V4__seed_organizer_data.sql** - Seeds initial organizer data
5. **V5__create_cigars_table.sql** - Creates the cigars table with all relationships
6. **V6__create_favorites_table.sql** - Creates the favorites table with snapshot fields
7. **V7__add_composite_indexes.sql** - Adds composite indexes for query optimization

## How It Works

The migration system is embedded in the application binary using Refinery's `embed_migrations!` macro:

```rust
// In src/main.rs
use refinery::embed_migrations;

// Embed migrations from the migrations directory
embed_migrations!("migrations");

// Run migrations on startup
let mut client = pool.get().await?;
migrations::runner().run_async(&mut **client).await?;
```

## Migration Features

### Composite Indexes (V7)

The latest migration adds several composite indexes for improved query performance:

- `idx_cigars_humidor_active` - Filters cigars by humidor and active status
- `idx_cigars_created_active` - Sorts cigars by creation date with active filter
- `idx_favorites_user_created` - Fetches user favorites sorted by creation date
- `idx_cigars_brand_active` - Partial index for active cigars with brands
- `idx_users_username_active` - Partial index for active users (faster login)
- `idx_humidors_user_created` - Fetches user humidors sorted by creation date

## Creating New Migrations

To create a new migration:

1. Create a new SQL file in the `migrations/` directory
2. Name it following the pattern: `V{next_number}__{description}.sql`
3. Write your migration SQL (use `IF NOT EXISTS` clauses for idempotency)
4. The migration will automatically run on next application start

Example:

```sql
-- migrations/V8__add_user_preferences.sql
CREATE TABLE IF NOT EXISTS user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    theme VARCHAR(50) DEFAULT 'dark',
    language VARCHAR(10) DEFAULT 'en',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);
```

## Best Practices

1. **Idempotent Migrations**: Use `IF NOT EXISTS` clauses
2. **Sequential Numbering**: Migrations are applied in version order (V1, V2, V3...)
3. **No Rollbacks**: Refinery doesn't support rollbacks - create a new forward migration
4. **Index Creation**: Use `CONCURRENTLY` for indexes on production databases
5. **Testing**: Test migrations on a development database before deploying

## Migration State

Refinery tracks applied migrations in the `refinery_schema_history` table:

```sql
SELECT * FROM refinery_schema_history ORDER BY version;
```

This table shows which migrations have been applied and when.

## Advantages Over Inline Migrations

1. **Version Control**: All schema changes are tracked in source control
2. **Reproducibility**: Database schema can be recreated from migrations
3. **Team Collaboration**: Easy to see what schema changes were made and when
4. **Rollout Control**: Migrations apply automatically on application start
5. **Audit Trail**: `refinery_schema_history` table tracks all applied migrations

## Troubleshooting

### Migration Fails

If a migration fails:

1. Check the error message in application logs
2. Verify SQL syntax in the migration file
3. Check database permissions
4. Look at `refinery_schema_history` to see which migrations succeeded

### Starting Fresh

To reset the database (development only):

```bash
# Drop and recreate database
docker-compose down -v
docker-compose up --build
```

This will apply all migrations from scratch.

## References

- [Refinery Documentation](https://github.com/rust-db/refinery)
- [PostgreSQL Index Documentation](https://www.postgresql.org/docs/current/indexes.html)
- [SQL Best Practices](https://wiki.postgresql.org/wiki/Don%27t_Do_This)
