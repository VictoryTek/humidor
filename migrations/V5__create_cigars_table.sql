-- Create cigars table with foreign keys to organizer tables
CREATE TABLE IF NOT EXISTS cigars (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    humidor_id UUID REFERENCES humidors(id) ON DELETE SET NULL,
    brand_id UUID REFERENCES brands(id) ON DELETE SET NULL,
    name VARCHAR NOT NULL,
    size_id UUID REFERENCES sizes(id) ON DELETE SET NULL,
    strength_id UUID REFERENCES strengths(id) ON DELETE SET NULL,
    origin_id UUID REFERENCES origins(id) ON DELETE SET NULL,
    wrapper VARCHAR,
    binder VARCHAR,
    filler VARCHAR,
    price DOUBLE PRECISION,
    purchase_date TIMESTAMPTZ,
    notes TEXT,
    quantity INTEGER NOT NULL DEFAULT 1,
    ring_gauge_id UUID REFERENCES ring_gauges(id) ON DELETE SET NULL,
    length DOUBLE PRECISION,
    image_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_cigars_brand_id ON cigars(brand_id);
CREATE INDEX IF NOT EXISTS idx_cigars_size_id ON cigars(size_id);
CREATE INDEX IF NOT EXISTS idx_cigars_origin_id ON cigars(origin_id);
CREATE INDEX IF NOT EXISTS idx_cigars_strength_id ON cigars(strength_id);
CREATE INDEX IF NOT EXISTS idx_cigars_ring_gauge_id ON cigars(ring_gauge_id);
CREATE INDEX IF NOT EXISTS idx_cigars_is_active ON cigars(is_active);
