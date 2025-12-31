-- Add columns for API fetching
ALTER TABLE assets ADD COLUMN IF NOT EXISTS api_ticker VARCHAR(50);
ALTER TABLE assets ADD COLUMN IF NOT EXISTS source VARCHAR(50) DEFAULT 'YAHOO';

-- Default existing assets to YAHOO/Same Ticker if not set
UPDATE assets SET api_ticker = ticker WHERE api_ticker IS NULL;

-- Seed Cryptocurrencies (Binance / CoinGecko)
INSERT INTO assets (ticker, name, asset_type, api_ticker, source) VALUES
('BTC', 'Bitcoin', 'Crypto', 'BTCUSDT', 'BINANCE'),
('ETH', 'Ethereum', 'Crypto', 'ETHUSDT', 'BINANCE'),
('BNB', 'Binance Coin', 'Crypto', 'BNBUSDT', 'BINANCE'),
('SOL', 'Solana', 'Crypto', 'SOLUSDT', 'BINANCE'),
('XRP', 'Ripple', 'Crypto', 'XRPUSDT', 'BINANCE'),
('ADA', 'Cardano', 'Crypto', 'ADAUSDT', 'BINANCE'),
('DOGE', 'Dogecoin', 'Crypto', 'DOGEUSDT', 'BINANCE'),
('TRX', 'TRON', 'Crypto', 'TRXUSDT', 'BINANCE'),
('DOT', 'Polkadot', 'Crypto', 'DOTUSDT', 'BINANCE'),
('MATIC', 'Polygon', 'Crypto', 'MATICUSDT', 'BINANCE'),
('UMBRA', 'Umbra Network', 'Crypto', 'umbra-network', 'COINGECKO')
ON CONFLICT (ticker) DO UPDATE SET 
    api_ticker = EXCLUDED.api_ticker,
    source = EXCLUDED.source,
    name = EXCLUDED.name,
    asset_type = EXCLUDED.asset_type;

-- Seed Top US Stocks (YAHOO) - A selection of major ones
INSERT INTO assets (ticker, name, asset_type, api_ticker, source) VALUES
('AAPL', 'Apple Inc.', 'Stock', 'AAPL', 'YAHOO'),
('MSFT', 'Microsoft Corporation', 'Stock', 'MSFT', 'YAHOO'),
('GOOGL', 'Alphabet Inc.', 'Stock', 'GOOGL', 'YAHOO'),
('AMZN', 'Amazon.com Inc.', 'Stock', 'AMZN', 'YAHOO'),
('NVDA', 'NVIDIA Corporation', 'Stock', 'NVDA', 'YAHOO'),
('TSLA', 'Tesla Inc.', 'Stock', 'TSLA', 'YAHOO'),
('META', 'Meta Platforms Inc.', 'Stock', 'META', 'YAHOO'),
('BRK.B', 'Berkshire Hathaway Inc.', 'Stock', 'BRK.B', 'YAHOO'),
('LLY', 'Eli Lilly and Company', 'Stock', 'LLY', 'YAHOO'),
('V', 'Visa Inc.', 'Stock', 'V', 'YAHOO'),
('TSM', 'Taiwan Semiconductor Manufacturing', 'Stock', 'TSM', 'YAHOO'),
('UNH', 'UnitedHealth Group', 'Stock', 'UNH', 'YAHOO'),
('XOM', 'Exxon Mobil Corporation', 'Stock', 'XOM', 'YAHOO'),
('JNJ', 'Johnson & Johnson', 'Stock', 'JNJ', 'YAHOO'),
('JPM', 'JPMorgan Chase & Co.', 'Stock', 'JPM', 'YAHOO'),
('WMT', 'Walmart Inc.', 'Stock', 'WMT', 'YAHOO'),
('MA', 'Mastercard Incorporated', 'Stock', 'MA', 'YAHOO'),
('PG', 'Procter & Gamble Company', 'Stock', 'PG', 'YAHOO'),
('AVGO', 'Broadcom Inc.', 'Stock', 'AVGO', 'YAHOO'),
('HD', 'The Home Depot', 'Stock', 'HD', 'YAHOO'),
('CVX', 'Chevron Corporation', 'Stock', 'CVX', 'YAHOO'),
('MRK', 'Merck & Co.', 'Stock', 'MRK', 'YAHOO'),
('ABBV', 'AbbVie Inc.', 'Stock', 'ABBV', 'YAHOO'),
('KO', 'The Coca-Cola Company', 'Stock', 'KO', 'YAHOO'),
('PEP', 'PepsiCo Inc.', 'Stock', 'PEP', 'YAHOO'),
('COST', 'Costco Wholesale', 'Stock', 'COST', 'YAHOO'),
('BAC', 'Bank of America', 'Stock', 'BAC', 'YAHOO'),
('ADBE', 'Adobe Inc.', 'Stock', 'ADBE', 'YAHOO'),
('CRM', 'Salesforce Inc.', 'Stock', 'CRM', 'YAHOO'),
('AMD', 'Advanced Micro Devices', 'Stock', 'AMD', 'YAHOO'),
('NFLX', 'Netflix Inc.', 'Stock', 'NFLX', 'YAHOO'),
('MCD', 'McDonald''s Corporation', 'Stock', 'MCD', 'YAHOO'),
('CSCO', 'Cisco Systems', 'Stock', 'CSCO', 'YAHOO'),
('INTC', 'Intel Corporation', 'Stock', 'INTC', 'YAHOO'),
('T', 'AT&T Inc.', 'Stock', 'T', 'YAHOO'),
('DIS', 'The Walt Disney Company', 'Stock', 'DIS', 'YAHOO'),
('NKE', 'Nike Inc.', 'Stock', 'NKE', 'YAHOO'),
('VZ', 'Verizon Communications', 'Stock', 'VZ', 'YAHOO'),
('CMCSA', 'Comcast Corporation', 'Stock', 'CMCSA', 'YAHOO'),
('PFE', 'Pfizer Inc.', 'Stock', 'PFE', 'YAHOO'),
('INTU', 'Intuit Inc.', 'Stock', 'INTU', 'YAHOO'),
('QCOM', 'Qualcomm Inc.', 'Stock', 'QCOM', 'YAHOO'),
('IBM', 'IBM', 'Stock', 'IBM', 'YAHOO'),
('AMGA', 'Amgen Inc.', 'Stock', 'AMGN', 'YAHOO'),
('TXN', 'Texas Instruments', 'Stock', 'TXN', 'YAHOO'),
('GE', 'General Electric', 'Stock', 'GE', 'YAHOO'),
('NOW', 'ServiceNow', 'Stock', 'NOW', 'YAHOO'),
('SPY', 'SPDR S&P 500 ETF Trust', 'Stock', 'SPY', 'YAHOO'),
('VOO', 'Vanguard S&P 500 ETF', 'Stock', 'VOO', 'YAHOO'),
('QQQ', 'Invesco QQQ Trust', 'Stock', 'QQQ', 'YAHOO')
-- Note: Truncated list for brevity, but this covers major assets.
ON CONFLICT (ticker) DO UPDATE SET 
    api_ticker = EXCLUDED.api_ticker,
    source = EXCLUDED.source,
    name = EXCLUDED.name,
    asset_type = EXCLUDED.asset_type;
