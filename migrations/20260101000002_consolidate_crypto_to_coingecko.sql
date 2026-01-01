-- Consolidate all crypto assets to use CoinGecko instead of Binance
-- Update source and api_ticker to match CoinGecko's coin IDs

UPDATE assets 
SET 
    source = 'COINGECKO',
    api_ticker = CASE ticker
        WHEN 'BTC' THEN 'bitcoin'
        WHEN 'ETH' THEN 'ethereum'
        WHEN 'BNB' THEN 'binancecoin'
        WHEN 'SOL' THEN 'solana'
        WHEN 'XRP' THEN 'ripple'
        WHEN 'ADA' THEN 'cardano'
        WHEN 'DOGE' THEN 'dogecoin'
        WHEN 'DOT' THEN 'polkadot'
        WHEN 'MATIC' THEN 'matic-network'
        WHEN 'AVAX' THEN 'avalanche-2'
        ELSE LOWER(ticker)
    END
WHERE source = 'BINANCE' OR asset_type = 'Crypto';
