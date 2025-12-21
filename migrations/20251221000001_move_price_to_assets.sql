-- Move current_price from portfolio to assets table

-- 1. Add columns to assets table
ALTER TABLE assets 
ADD COLUMN current_price DECIMAL(19, 4),
ADD COLUMN last_updated TIMESTAMPTZ DEFAULT NOW();

-- 2. Migrate existing data (Optional: depends if we want to keep stale data, but good for continuity)
-- We can take the latest price found in portfolio for each ticker.
-- Since portfolio had user-specific rows, multiple users might have different 'current_price' for the same ticker (race condition).
-- We'll pick one (e.g., max or average, or just any). Let's maximize it to be safe or just take one.
UPDATE assets a
SET current_price = p.price
FROM (
    SELECT ticker, MAX(current_price) as price
    FROM portfolio
    GROUP BY ticker
) p
WHERE a.ticker = p.ticker;

-- 3. Drop columns from portfolio table
ALTER TABLE portfolio 
DROP COLUMN current_price,
DROP COLUMN last_updated;
