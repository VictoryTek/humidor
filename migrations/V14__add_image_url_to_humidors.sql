-- Add image_url column to humidors table for storing humidor images
ALTER TABLE humidors ADD COLUMN IF NOT EXISTS image_url TEXT;

-- Add comment for documentation
COMMENT ON COLUMN humidors.image_url IS 'URL or path to humidor image (external URL or uploaded file path)';
