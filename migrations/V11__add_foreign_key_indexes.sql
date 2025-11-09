-- Add indexes for foreign key columns to improve query performance
-- These indexes speed up JOIN operations and WHERE clauses filtering by foreign keys

-- Indexes for cigars table foreign keys (if not already exists from V7)
CREATE INDEX IF NOT EXISTS idx_cigars_brand_id ON cigars(brand_id);
CREATE INDEX IF NOT EXISTS idx_cigars_size_id ON cigars(size_id);
CREATE INDEX IF NOT EXISTS idx_cigars_origin_id ON cigars(origin_id);
CREATE INDEX IF NOT EXISTS idx_cigars_strength_id ON cigars(strength_id);
CREATE INDEX IF NOT EXISTS idx_cigars_ring_gauge_id ON cigars(ring_gauge_id);

-- Additional composite index for common filter combinations
CREATE INDEX IF NOT EXISTS idx_cigars_humidor_brand ON cigars(humidor_id, brand_id) WHERE is_active = true;

-- Index for sorting cigars by multiple criteria (active status, then date)
CREATE INDEX IF NOT EXISTS idx_cigars_active_date ON cigars(is_active DESC, created_at DESC);

-- Indexes for wish_list (if not already created in V8)
CREATE INDEX IF NOT EXISTS idx_wish_list_user_id ON wish_list(user_id);
CREATE INDEX IF NOT EXISTS idx_wish_list_cigar_id ON wish_list(cigar_id);
CREATE INDEX IF NOT EXISTS idx_wish_list_user_created ON wish_list(user_id, created_at DESC);

-- Index for favorites with composite key (if not already exists from V6)
CREATE INDEX IF NOT EXISTS idx_favorites_cigar_id ON favorites(cigar_id);

-- Index for password reset tokens expiration cleanup
CREATE INDEX IF NOT EXISTS idx_password_reset_created ON password_reset_tokens(created_at);

-- Index for user email lookups (commonly used in auth)
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email) WHERE is_active = true;
