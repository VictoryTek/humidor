-- Add image_url field to cigars table
ALTER TABLE cigars ADD COLUMN IF NOT EXISTS image_url TEXT;
