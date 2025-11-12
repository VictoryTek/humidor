-- Create humidor_shares table for sharing humidors between users
-- This allows users to grant different permission levels to other users

CREATE TABLE IF NOT EXISTS humidor_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    humidor_id UUID NOT NULL REFERENCES humidors(id) ON DELETE CASCADE,
    shared_with_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    shared_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permission_level VARCHAR(20) NOT NULL CHECK (permission_level IN ('view', 'edit', 'full')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Prevent duplicate shares (same humidor shared with same user multiple times)
    CONSTRAINT unique_humidor_share UNIQUE (humidor_id, shared_with_user_id)
);

-- Create indexes for efficient querying
CREATE INDEX idx_humidor_shares_humidor_id ON humidor_shares(humidor_id);
CREATE INDEX idx_humidor_shares_shared_with_user_id ON humidor_shares(shared_with_user_id);
CREATE INDEX idx_humidor_shares_shared_by_user_id ON humidor_shares(shared_by_user_id);

-- Add comments for documentation
COMMENT ON TABLE humidor_shares IS 'Tracks which users have access to which humidors and their permission levels';
COMMENT ON COLUMN humidor_shares.permission_level IS 'Permission level: view (read-only), edit (add/edit cigars), full (add/edit/delete cigars and manage shares)';
COMMENT ON COLUMN humidor_shares.shared_with_user_id IS 'The user who is being granted access';
COMMENT ON COLUMN humidor_shares.shared_by_user_id IS 'The user who granted the access (usually the owner)';
