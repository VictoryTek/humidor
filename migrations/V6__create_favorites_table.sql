-- Create favorites table with snapshot fields for preserving cigar data
CREATE TABLE IF NOT EXISTS favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cigar_id UUID REFERENCES cigars(id) ON DELETE SET NULL,
    snapshot_name TEXT,
    snapshot_brand_id UUID,
    snapshot_size_id UUID,
    snapshot_strength_id UUID,
    snapshot_origin_id UUID,
    snapshot_ring_gauge_id UUID,
    snapshot_image_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, cigar_id)
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_favorites_user_id ON favorites(user_id);
CREATE INDEX IF NOT EXISTS idx_favorites_cigar_id ON favorites(cigar_id);
