-- Fix migration: Update currency for IDX and SGX stocks that were inserted without correct currency
-- This fixes assets that already exist in the database with wrong currency (USD instead of IDR/SGD)

-- Fix Indonesian stocks to use IDR
UPDATE assets SET currency = 'IDR' WHERE ticker LIKE '%.JK';

-- Fix Singapore stocks to use SGD  
UPDATE assets SET currency = 'SGD' WHERE ticker LIKE '%.SI';
