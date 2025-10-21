-- Create organizer tables for managing cigar attributes

-- Brands table
CREATE TABLE brands (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL UNIQUE,
    description TEXT,
    country VARCHAR,
    website VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sizes table
CREATE TABLE sizes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL UNIQUE,
    length_inches DECIMAL(4,2),
    ring_gauge INTEGER,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Origins table
CREATE TABLE origins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL UNIQUE,
    country VARCHAR NOT NULL,
    region VARCHAR,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Strengths table
CREATE TABLE strengths (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL UNIQUE,
    level INTEGER NOT NULL, -- 1=Mild, 2=Medium-Mild, 3=Medium, 4=Medium-Full, 5=Full
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Ring Gauges table
CREATE TABLE ring_gauges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    gauge INTEGER NOT NULL UNIQUE,
    description TEXT,
    common_names VARCHAR[], -- Array of common names for this gauge
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_brands_name ON brands(name);
CREATE INDEX idx_brands_country ON brands(country);
CREATE INDEX idx_sizes_name ON sizes(name);
CREATE INDEX idx_sizes_ring_gauge ON sizes(ring_gauge);
CREATE INDEX idx_origins_name ON origins(name);
CREATE INDEX idx_origins_country ON origins(country);
CREATE INDEX idx_strengths_name ON strengths(name);
CREATE INDEX idx_strengths_level ON strengths(level);
CREATE INDEX idx_ring_gauges_gauge ON ring_gauges(gauge);

-- Insert some default strength values
INSERT INTO strengths (name, level, description) VALUES
('Mild', 1, 'Light and smooth, perfect for beginners'),
('Medium-Mild', 2, 'Slightly more body than mild, still approachable'),
('Medium', 3, 'Balanced strength with good complexity'),
('Medium-Full', 4, 'Strong flavor with substantial body'),
('Full', 5, 'Bold and intense, for experienced smokers');

-- Insert some common ring gauges
INSERT INTO ring_gauges (gauge, description, common_names) VALUES
(38, 'Very thin gauge, quick smoke', ARRAY['Lancero thin', 'Panetela']),
(42, 'Classic thin gauge', ARRAY['Corona', 'Petit Corona']),
(44, 'Standard corona size', ARRAY['Corona', 'Lonsdale']),
(46, 'Popular medium gauge', ARRAY['Corona Gorda', 'Petit Robusto']),
(48, 'Medium-thick gauge', ARRAY['Robusto thin']),
(50, 'Classic robusto gauge', ARRAY['Robusto', 'Rothschild']),
(52, 'Thick robusto gauge', ARRAY['Robusto Extra', 'Toro thin']),
(54, 'Toro gauge', ARRAY['Toro', 'Gordo']),
(56, 'Churchill gauge', ARRAY['Churchill', 'Double Corona']),
(58, 'Thick churchill', ARRAY['Churchill Extra']),
(60, 'Very thick gauge', ARRAY['Gordo', 'Double Toro']);