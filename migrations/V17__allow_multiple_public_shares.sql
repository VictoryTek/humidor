-- Remove the unique constraint that limits one public share per humidor
-- This allows creating multiple public shares with different expiration dates and settings

ALTER TABLE public_humidor_shares
DROP CONSTRAINT IF EXISTS unique_public_humidor_share;

-- Add a label/description field to help distinguish between multiple shares
ALTER TABLE public_humidor_shares
ADD COLUMN IF NOT EXISTS label VARCHAR(100);

COMMENT ON COLUMN public_humidor_shares.label IS 'Optional label to help identify the purpose of this share (e.g., "Share for John - Expires 2025-12-15")';
