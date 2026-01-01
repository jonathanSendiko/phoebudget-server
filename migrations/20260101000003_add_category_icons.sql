-- Add icon column to categories
ALTER TABLE categories ADD COLUMN IF NOT EXISTS icon VARCHAR(50);

-- Populate icons for existing seed data
UPDATE categories SET icon = 'restaurant' WHERE name = 'Food';
UPDATE categories SET icon = 'commute' WHERE name = 'Transport';
UPDATE categories SET icon = 'attach_money' WHERE name = 'Salary';
UPDATE categories SET icon = 'movie' WHERE name = 'Entertainment';
UPDATE categories SET icon = 'shopping_bag' WHERE name = 'Shopping';
UPDATE categories SET icon = 'health_and_safety' WHERE name = 'Healthcare';
UPDATE categories SET icon = 'school' WHERE name = 'Education';
UPDATE categories SET icon = 'bolt' WHERE name = 'Utilities';
UPDATE categories SET icon = 'home' WHERE name = 'Housing';
UPDATE categories SET icon = 'security' WHERE name = 'Insurance';
UPDATE categories SET icon = 'self_improvement' WHERE name = 'Personal Care';
UPDATE categories SET icon = 'work' WHERE name = 'Freelance';
UPDATE categories SET icon = 'trending_up' WHERE name = 'Investment Returns';

-- Set default for any missing entries
UPDATE categories SET icon = 'help_outline' WHERE icon IS NULL;
