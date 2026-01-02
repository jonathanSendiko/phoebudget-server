-- Add flag to exclude specific categories from spending analysis
ALTER TABLE categories ADD COLUMN exclude_from_analysis BOOLEAN DEFAULT FALSE;

-- Insert Transfer categories (one for money leaving a pocket, one for coming in)
INSERT INTO categories (name, is_income, exclude_from_analysis) VALUES 
('Transfer Out', false, true),
('Transfer In', true, true);
