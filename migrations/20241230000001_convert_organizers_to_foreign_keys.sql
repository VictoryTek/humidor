-- Migration: Convert organizer text fields to foreign key relationships
-- This migration converts the cigars table from storing organizer names as VARCHAR
-- to using proper UUID foreign keys referencing the organizer tables

-- Step 1: Add new foreign key columns
ALTER TABLE cigars ADD COLUMN brand_id UUID;
ALTER TABLE cigars ADD COLUMN size_id UUID;
ALTER TABLE cigars ADD COLUMN origin_id UUID;
ALTER TABLE cigars ADD COLUMN strength_id UUID;
ALTER TABLE cigars ADD COLUMN ring_gauge_id UUID;

-- Step 2: Migrate existing data by matching names (case-insensitive)
-- Update brand_id based on brand name
UPDATE cigars c
SET brand_id = b.id
FROM brands b
WHERE LOWER(TRIM(c.brand)) = LOWER(TRIM(b.name));

-- Update size_id based on size name
UPDATE cigars c
SET size_id = s.id
FROM sizes s
WHERE LOWER(TRIM(c.size)) = LOWER(TRIM(s.name));

-- Update origin_id based on origin name
UPDATE cigars c
SET origin_id = o.id
FROM origins o
WHERE LOWER(TRIM(c.origin)) = LOWER(TRIM(o.name));

-- Update strength_id based on strength name
UPDATE cigars c
SET strength_id = st.id
FROM strengths st
WHERE LOWER(TRIM(c.strength)) = LOWER(TRIM(st.name));

-- Update ring_gauge_id based on ring_gauge value
UPDATE cigars c
SET ring_gauge_id = rg.id
FROM ring_gauges rg
WHERE c.ring_gauge = rg.gauge;

-- Step 3: Drop the old VARCHAR columns
ALTER TABLE cigars DROP COLUMN brand;
ALTER TABLE cigars DROP COLUMN size;
ALTER TABLE cigars DROP COLUMN origin;
ALTER TABLE cigars DROP COLUMN strength;
ALTER TABLE cigars DROP COLUMN ring_gauge;

-- Step 4: Add foreign key constraints
ALTER TABLE cigars 
    ADD CONSTRAINT fk_cigars_brand 
    FOREIGN KEY (brand_id) 
    REFERENCES brands(id) 
    ON DELETE SET NULL;

ALTER TABLE cigars 
    ADD CONSTRAINT fk_cigars_size 
    FOREIGN KEY (size_id) 
    REFERENCES sizes(id) 
    ON DELETE SET NULL;

ALTER TABLE cigars 
    ADD CONSTRAINT fk_cigars_origin 
    FOREIGN KEY (origin_id) 
    REFERENCES origins(id) 
    ON DELETE SET NULL;

ALTER TABLE cigars 
    ADD CONSTRAINT fk_cigars_strength 
    FOREIGN KEY (strength_id) 
    REFERENCES strengths(id) 
    ON DELETE SET NULL;

ALTER TABLE cigars 
    ADD CONSTRAINT fk_cigars_ring_gauge 
    FOREIGN KEY (ring_gauge_id) 
    REFERENCES ring_gauges(id) 
    ON DELETE SET NULL;

-- Step 5: Add indexes for better query performance
CREATE INDEX idx_cigars_brand_id ON cigars(brand_id);
CREATE INDEX idx_cigars_size_id ON cigars(size_id);
CREATE INDEX idx_cigars_origin_id ON cigars(origin_id);
CREATE INDEX idx_cigars_strength_id ON cigars(strength_id);
CREATE INDEX idx_cigars_ring_gauge_id ON cigars(ring_gauge_id);
