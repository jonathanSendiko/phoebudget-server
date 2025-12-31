-- Add additional spending and income categories
-- 8 spending categories + 2 income categories

INSERT INTO categories (name, is_income) VALUES 
-- Spending categories
('Entertainment', false),
('Shopping', false),
('Healthcare', false),
('Education', false),
('Utilities', false),
('Housing', false),
('Insurance', false),
('Personal Care', false),

-- Income categories
('Freelance', true),
('Investment Returns', true);
