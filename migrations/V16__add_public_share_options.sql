-- Add options for what to include in public share
ALTER TABLE public_humidor_shares 
ADD COLUMN IF NOT EXISTS include_favorites BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN IF NOT EXISTS include_wish_list BOOLEAN NOT NULL DEFAULT false;

-- Add index for filtering
CREATE INDEX IF NOT EXISTS idx_public_shares_options ON public_humidor_shares(include_favorites, include_wish_list);

COMMENT ON COLUMN public_humidor_shares.include_favorites IS 'Whether to show favorites in the public share';
COMMENT ON COLUMN public_humidor_shares.include_wish_list IS 'Whether to show wish list in the public share';
