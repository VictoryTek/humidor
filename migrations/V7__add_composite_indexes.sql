-- Composite indexes for better query performance
-- These indexes improve performance for common query patterns
-- Note: CONCURRENTLY keyword removed because migrations run inside transactions

-- Index for filtering cigars by humidor and active status
CREATE INDEX IF NOT EXISTS idx_cigars_humidor_active 
ON cigars(humidor_id, is_active);

-- Index for sorting cigars by creation date with active filter
CREATE INDEX IF NOT EXISTS idx_cigars_created_active 
ON cigars(created_at DESC, is_active);

-- Index for fetching user favorites sorted by creation date
CREATE INDEX IF NOT EXISTS idx_favorites_user_created 
ON favorites(user_id, created_at DESC);

-- Partial index for active cigars with brand (excluding NULL brands)
CREATE INDEX IF NOT EXISTS idx_cigars_brand_active 
ON cigars(brand_id, is_active) WHERE brand_id IS NOT NULL;

-- Partial index for active users (faster login lookups)
CREATE INDEX IF NOT EXISTS idx_users_username_active 
ON users(username) WHERE is_active = true;

-- Index for fetching user humidors sorted by creation date
CREATE INDEX IF NOT EXISTS idx_humidors_user_created 
ON humidors(user_id, created_at DESC);
