CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE cigars (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    brand VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    size VARCHAR NOT NULL,
    strength VARCHAR NOT NULL,
    origin VARCHAR NOT NULL,
    wrapper VARCHAR,
    binder VARCHAR,
    filler VARCHAR,
    price DECIMAL(10,2),
    purchase_date TIMESTAMPTZ,
    notes TEXT,
    quantity INTEGER NOT NULL DEFAULT 1,
    humidor_location VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cigars_brand ON cigars(brand);
CREATE INDEX idx_cigars_strength ON cigars(strength);
CREATE INDEX idx_cigars_origin ON cigars(origin);
CREATE INDEX idx_cigars_created_at ON cigars(created_at);