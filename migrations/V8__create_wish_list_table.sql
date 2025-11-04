-- Create wish_list table (similar to favorites)
CREATE TABLE IF NOT EXISTS wish_list (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cigar_id UUID NOT NULL REFERENCES cigars(id) ON DELETE CASCADE,
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, cigar_id)
);

-- Add index for faster lookups
CREATE INDEX idx_wish_list_user_id ON wish_list(user_id);
CREATE INDEX idx_wish_list_cigar_id ON wish_list(cigar_id);
CREATE INDEX idx_wish_list_created ON wish_list(created_at DESC);
