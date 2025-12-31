-- Add currency support columns to transactions table
ALTER TABLE transactions 
ADD COLUMN original_currency VARCHAR(3),
ADD COLUMN original_amount DECIMAL(19, 4),
ADD COLUMN exchange_rate DECIMAL(19, 8);
