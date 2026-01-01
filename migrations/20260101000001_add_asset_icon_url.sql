-- Add icon_url column to assets table for storing logo URLs
-- Icons will be fetched from CoinGecko API for cryptocurrency assets

ALTER TABLE assets ADD COLUMN IF NOT EXISTS icon_url TEXT;
