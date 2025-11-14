-- Add user_id to all organizer tables to make them user-specific

-- Delete existing seed data (will be re-added as user-specific data in setup)
DELETE FROM brands;
DELETE FROM sizes;
DELETE FROM origins;
DELETE FROM strengths;
DELETE FROM ring_gauges;

-- Add user_id to brands table
ALTER TABLE brands ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE;

-- Add user_id to sizes table
ALTER TABLE sizes ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE;

-- Add user_id to origins table
ALTER TABLE origins ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE;

-- Add user_id to strengths table
ALTER TABLE strengths ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE;

-- Add user_id to ring_gauges table
ALTER TABLE ring_gauges ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE;

-- Drop the old unique constraints on name/gauge
ALTER TABLE brands DROP CONSTRAINT IF EXISTS brands_name_key;
ALTER TABLE sizes DROP CONSTRAINT IF EXISTS sizes_name_key;
ALTER TABLE origins DROP CONSTRAINT IF EXISTS origins_name_key;
ALTER TABLE strengths DROP CONSTRAINT IF EXISTS strengths_name_key;
ALTER TABLE ring_gauges DROP CONSTRAINT IF EXISTS ring_gauges_gauge_key;

-- Add new composite unique constraints (user_id + name/gauge)
ALTER TABLE brands ADD CONSTRAINT brands_user_id_name_key UNIQUE (user_id, name);
ALTER TABLE sizes ADD CONSTRAINT sizes_user_id_name_key UNIQUE (user_id, name);
ALTER TABLE origins ADD CONSTRAINT origins_user_id_name_key UNIQUE (user_id, name);
ALTER TABLE strengths ADD CONSTRAINT strengths_user_id_name_key UNIQUE (user_id, name);
ALTER TABLE ring_gauges ADD CONSTRAINT ring_gauges_user_id_gauge_key UNIQUE (user_id, gauge);

-- Add indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_brands_user_id ON brands(user_id);
CREATE INDEX IF NOT EXISTS idx_sizes_user_id ON sizes(user_id);
CREATE INDEX IF NOT EXISTS idx_origins_user_id ON origins(user_id);
CREATE INDEX IF NOT EXISTS idx_strengths_user_id ON strengths(user_id);
CREATE INDEX IF NOT EXISTS idx_ring_gauges_user_id ON ring_gauges(user_id);
