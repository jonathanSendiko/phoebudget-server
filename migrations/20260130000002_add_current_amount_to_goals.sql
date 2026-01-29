-- Add current_amount column to financial_goals table
ALTER TABLE financial_goals ADD COLUMN current_amount DECIMAL(15,2) NOT NULL DEFAULT 0.00;
