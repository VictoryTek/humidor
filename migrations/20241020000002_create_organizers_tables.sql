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
    length_inches DOUBLE PRECISION,
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

-- Insert common cigar brands
INSERT INTO brands (name, description, country) VALUES
('Arturo Fuente', 'Premium Dominican cigars, known for OpusX and Hemingway lines', 'Dominican Republic'),
('Davidoff', 'Luxury Swiss brand with premium tobacco', 'Switzerland'),
('Padron', 'Family-owned Nicaraguan brand known for quality and consistency', 'Nicaragua'),
('Cohiba', 'Iconic Cuban brand, flagship of Habanos', 'Cuba'),
('Montecristo', 'One of the most recognized Cuban brands worldwide', 'Cuba'),
('Romeo y Julieta', 'Classic Cuban brand with wide variety', 'Cuba'),
('Partagas', 'Historic Cuban brand known for full-bodied cigars', 'Cuba'),
('Hoyo de Monterrey', 'Cuban brand known for mild to medium strength', 'Cuba'),
('Oliva', 'Nicaraguan family business with consistent quality', 'Nicaragua'),
('My Father', 'Premium Nicaraguan brand by Jose "Pepin" Garcia', 'Nicaragua'),
('Drew Estate', 'Innovative American brand, makers of Liga Privada and Acid', 'United States'),
('Rocky Patel', 'Popular brand with wide range of blends', 'Honduras'),
('Ashton', 'Premium brand with Dominican and Nicaraguan lines', 'United States'),
('Alec Bradley', 'Honduran brand known for Prensado and Black Market', 'Honduras'),
('La Flor Dominicana', 'Dominican brand known for powerful cigars', 'Dominican Republic'),
('Perdomo', 'Nicaraguan brand with extensive aging program', 'Nicaragua'),
('Tatuaje', 'Boutique brand known for Nicaraguan puros', 'Nicaragua'),
('Liga Privada', 'Premium line from Drew Estate', 'United States'),
('Punch', 'Cuban brand known for robust flavors', 'Cuba'),
('H. Upmann', 'Historic Cuban brand dating to 1844', 'Cuba');

-- Insert common cigar origins
INSERT INTO origins (name, country, region, description) VALUES
('Cuba', 'Cuba', NULL, 'Historic birthplace of premium cigars, known for rich flavor profiles'),
('Dominican Republic', 'Dominican Republic', NULL, 'World''s largest cigar producer, known for smooth, mild to medium cigars'),
('Nicaragua', 'Nicaragua', NULL, 'Produces full-bodied, peppery cigars with bold flavors'),
('Honduras', 'Honduras', NULL, 'Known for robust, flavorful cigars with Cuban-seed tobacco'),
('Mexico', 'Mexico', NULL, 'Produces rich, earthy cigars with quality wrapper tobacco'),
('United States', 'United States', NULL, 'Home to premium brands and innovative blends'),
('Ecuador', 'Ecuador', NULL, 'Famous for high-quality Connecticut Shade wrapper tobacco'),
('Brazil', 'Brazil', NULL, 'Known for dark, sweet maduro wrapper leaves'),
('Peru', 'Peru', NULL, 'Emerging origin with quality tobacco production'),
('Costa Rica', 'Costa Rica', NULL, 'Produces mild, smooth cigars with balanced flavor'),
('Panama', 'Panama', NULL, 'Small production of premium boutique cigars'),
('Colombia', 'Colombia', NULL, 'Growing reputation for quality tobacco'),
('Philippines', 'Philippines', NULL, 'Historic cigar production, value-priced offerings'),
('Indonesia', 'Indonesia', NULL, 'Known for Sumatra wrapper tobacco');

-- Insert common cigar sizes
INSERT INTO sizes (name, length_inches, ring_gauge, description) VALUES
('Petit Corona', 4.5, 42, 'Small classic size, 30-40 minute smoke'),
('Corona', 5.5, 42, 'Traditional Cuban size, balanced proportions'),
('Corona Gorda', 5.625, 46, 'Larger corona with more body'),
('Petit Robusto', 4.0, 50, 'Short and thick, concentrated flavor'),
('Robusto', 5.0, 50, 'Most popular size, 45-60 minute smoke'),
('Robusto Extra', 5.5, 50, 'Longer robusto for extended enjoyment'),
('Toro', 6.0, 50, 'Popular modern size, well-balanced'),
('Gordo', 6.0, 60, 'Large ring gauge, cooler smoke'),
('Churchill', 7.0, 47, 'Named after Winston Churchill, elegant size'),
('Double Corona', 7.5, 50, 'Large premium size, 90+ minute smoke'),
('Lancero', 7.5, 38, 'Long and thin, concentrated flavors'),
('Panetela', 6.0, 34, 'Slim and elegant, quick smoke'),
('Lonsdale', 6.5, 42, 'Classic thin vitola, refined smoke'),
('Torpedo', 6.125, 52, 'Tapered head, concentrated flavors'),
('Belicoso', 5.0, 52, 'Short pyramid shape with tapered head'),
('Perfecto', 5.0, 48, 'Tapered at both ends, unique experience'),
('Presidente', 8.0, 50, 'Extra-long premium size'),
('Rothschild', 4.5, 50, 'Short robusto, rich and quick'),
('Corona Extra', 5.5, 46, 'Medium size with good balance'),
('Gigante', 9.0, 52, 'Exceptionally large, 2+ hour smoke');