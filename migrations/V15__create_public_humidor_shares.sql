-- Create public_humidor_shares table for anonymous public sharing
-- Allows generating shareable links that work without authentication

CREATE TABLE IF NOT EXISTS public_humidor_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    humidor_id UUID NOT NULL REFERENCES humidors(id) ON DELETE CASCADE,
    created_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ DEFAULT (NOW() + INTERVAL '30 days'), -- Defaults to 30 days, NULL = permanent/no expiration
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Prevent multiple public shares per humidor (one active public share at a time)
    CONSTRAINT unique_public_humidor_share UNIQUE (humidor_id)
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_public_shares_humidor ON public_humidor_shares(humidor_id);
CREATE INDEX IF NOT EXISTS idx_public_shares_expiry ON public_humidor_shares(expires_at) WHERE expires_at IS NOT NULL;

-- Add comments for documentation
COMMENT ON TABLE public_humidor_shares IS 'Public share tokens for anonymous read-only humidor access via shareable links';
COMMENT ON COLUMN public_humidor_shares.id IS 'UUID serves as both primary key and public access token in the shareable URL';
COMMENT ON COLUMN public_humidor_shares.expires_at IS 'Expiration timestamp. Defaults to 30 days from creation. NULL means permanent until manually revoked';
COMMENT ON COLUMN public_humidor_shares.created_by_user_id IS 'User who created the public share (usually the humidor owner)';
