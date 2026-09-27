-- Smoking journal: one row per "smoked this cigar" event, logged by the user who smoked it.
-- user_id is the logger, not necessarily the cigar's owner (a user with edit access to a shared
-- humidor can log a session too), so sessions on a shared collection stay attributable per-logger.
CREATE TABLE IF NOT EXISTS smoking_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cigar_id UUID NOT NULL REFERENCES cigars(id) ON DELETE CASCADE,
    smoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rating SMALLINT CHECK (rating IS NULL OR rating BETWEEN 1 AND 5),
    duration_minutes INTEGER CHECK (duration_minutes IS NULL OR duration_minutes > 0),
    pairing TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_smoking_sessions_user_id ON smoking_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_smoking_sessions_cigar_id ON smoking_sessions(cigar_id);
CREATE INDEX IF NOT EXISTS idx_smoking_sessions_smoked_at ON smoking_sessions(smoked_at DESC);
