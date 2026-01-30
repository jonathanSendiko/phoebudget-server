-- Migration: Update existing stock assets to use iTick as source and add IDX/SGX stocks
-- This migration:
-- 1. Updates all existing YAHOO stock sources to ITICK
-- 2. Adds top 20 Indonesian (IDX) stocks with IDR currency
-- 3. Adds top 20 Singapore (SGX) stocks with SGD currency

-- Step 1: Update existing YAHOO stocks to use ITICK
-- Note: REGION:CODE format is used, defaulting US stocks to just the ticker (region=US assumed)
UPDATE assets 
SET source = 'ITICK' 
WHERE source = 'YAHOO' 
  AND asset_type = 'Stock';

-- Step 2: Add Indonesian (IDX) Top 20 Stocks
-- Ticker format: ID:CODE (e.g., ID:BBCA), Currency: IDR
INSERT INTO assets (ticker, name, asset_type, api_ticker, source, currency) VALUES
('BBCA.JK', 'Bank Central Asia', 'Stock', 'ID:BBCA', 'ITICK', 'IDR'),
('BMRI.JK', 'Bank Mandiri', 'Stock', 'ID:BMRI', 'ITICK', 'IDR'),
('BBRI.JK', 'Bank Rakyat Indonesia', 'Stock', 'ID:BBRI', 'ITICK', 'IDR'),
('BBNI.JK', 'Bank Negara Indonesia', 'Stock', 'ID:BBNI', 'ITICK', 'IDR'),
('TLKM.JK', 'Telkom Indonesia', 'Stock', 'ID:TLKM', 'ITICK', 'IDR'),
('ASII.JK', 'Astra International', 'Stock', 'ID:ASII', 'ITICK', 'IDR'),
('UNVR.JK', 'Unilever Indonesia', 'Stock', 'ID:UNVR', 'ITICK', 'IDR'),
('ICBP.JK', 'Indofood CBP Sukses Makmur', 'Stock', 'ID:ICBP', 'ITICK', 'IDR'),
('INDF.JK', 'Indofood Sukses Makmur', 'Stock', 'ID:INDF', 'ITICK', 'IDR'),
('GGRM.JK', 'Gudang Garam', 'Stock', 'ID:GGRM', 'ITICK', 'IDR'),
('HMSP.JK', 'HM Sampoerna', 'Stock', 'ID:HMSP', 'ITICK', 'IDR'),
('KLBF.JK', 'Kalbe Farma', 'Stock', 'ID:KLBF', 'ITICK', 'IDR'),
('SMGR.JK', 'Semen Indonesia', 'Stock', 'ID:SMGR', 'ITICK', 'IDR'),
('ANTM.JK', 'Aneka Tambang', 'Stock', 'ID:ANTM', 'ITICK', 'IDR'),
('PTBA.JK', 'Bukit Asam', 'Stock', 'ID:PTBA', 'ITICK', 'IDR'),
('ADRO.JK', 'Adaro Energy', 'Stock', 'ID:ADRO', 'ITICK', 'IDR'),
('TBIG.JK', 'Tower Bersama Infrastructure', 'Stock', 'ID:TBIG', 'ITICK', 'IDR'),
('TOWR.JK', 'Sarana Menara Nusantara', 'Stock', 'ID:TOWR', 'ITICK', 'IDR'),
('EMTK.JK', 'Elang Mahkota Teknologi', 'Stock', 'ID:EMTK', 'ITICK', 'IDR'),
('MDKA.JK', 'Merdeka Copper Gold', 'Stock', 'ID:MDKA', 'ITICK', 'IDR')
ON CONFLICT (ticker) DO UPDATE SET 
    api_ticker = EXCLUDED.api_ticker,
    source = EXCLUDED.source,
    name = EXCLUDED.name,
    asset_type = EXCLUDED.asset_type,
    currency = EXCLUDED.currency;

-- Step 3: Add Singapore (SGX) Top 20 Stocks
-- Ticker format: SG:CODE (e.g., SG:D05), Currency: SGD
INSERT INTO assets (ticker, name, asset_type, api_ticker, source, currency) VALUES
('D05.SI', 'DBS Group Holdings', 'Stock', 'SG:D05', 'ITICK', 'SGD'),
('O39.SI', 'OCBC Bank', 'Stock', 'SG:O39', 'ITICK', 'SGD'),
('U11.SI', 'United Overseas Bank', 'Stock', 'SG:U11', 'ITICK', 'SGD'),
('Z74.SI', 'Singtel', 'Stock', 'SG:Z74', 'ITICK', 'SGD'),
('A17U.SI', 'CapitaLand Ascendas REIT', 'Stock', 'SG:A17U', 'ITICK', 'SGD'),
('C09.SI', 'City Developments', 'Stock', 'SG:C09', 'ITICK', 'SGD'),
('C38U.SI', 'CapitaLand Integrated Commercial Trust', 'Stock', 'SG:C38U', 'ITICK', 'SGD'),
('F34.SI', 'Wilmar International', 'Stock', 'SG:F34', 'ITICK', 'SGD'),
('G13.SI', 'Genting Singapore', 'Stock', 'SG:G13', 'ITICK', 'SGD'),
('H78.SI', 'Hongkong Land Holdings', 'Stock', 'SG:H78', 'ITICK', 'SGD'),
('BN4.SI', 'Keppel Corporation', 'Stock', 'SG:BN4', 'ITICK', 'SGD'),
('N2IU.SI', 'Mapletree Pan Asia Commercial Trust', 'Stock', 'SG:N2IU', 'ITICK', 'SGD'),
('ME8U.SI', 'Mapletree Industrial Trust', 'Stock', 'SG:ME8U', 'ITICK', 'SGD'),
('M44U.SI', 'Mapletree Logistics Trust', 'Stock', 'SG:M44U', 'ITICK', 'SGD'),
('S63.SI', 'Singapore Technologies Engineering', 'Stock', 'SG:S63', 'ITICK', 'SGD'),
('S68.SI', 'Singapore Exchange', 'Stock', 'SG:S68', 'ITICK', 'SGD'),
('V03.SI', 'Venture Corporation', 'Stock', 'SG:V03', 'ITICK', 'SGD'),
('Y92.SI', 'Thai Beverage', 'Stock', 'SG:Y92', 'ITICK', 'SGD'),
('U96.SI', 'Sembcorp Industries', 'Stock', 'SG:U96', 'ITICK', 'SGD'),
('BS6.SI', 'YZJ Shipbuilding', 'Stock', 'SG:BS6', 'ITICK', 'SGD')
ON CONFLICT (ticker) DO UPDATE SET 
    api_ticker = EXCLUDED.api_ticker,
    source = EXCLUDED.source,
    name = EXCLUDED.name,
    asset_type = EXCLUDED.asset_type,
    currency = EXCLUDED.currency;
