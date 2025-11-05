-- Add retail_link column to cigars table
ALTER TABLE cigars ADD COLUMN IF NOT EXISTS retail_link TEXT;

-- Add index for retail_link queries
CREATE INDEX IF NOT EXISTS idx_cigars_retail_link ON cigars(retail_link) WHERE retail_link IS NOT NULL;
