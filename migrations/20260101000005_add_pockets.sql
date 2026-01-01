-- Create pockets table
CREATE TABLE pockets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    name VARCHAR(50) NOT NULL,
    description TEXT,
    icon VARCHAR(50) DEFAULT 'account_balance_wallet',
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Add unique constraint: one default pocket per user
CREATE UNIQUE INDEX unique_default_pocket_per_user 
ON pockets (user_id) WHERE is_default = TRUE;

-- Add pocket_id to transactions (nullable initially for migration)
ALTER TABLE transactions ADD COLUMN pocket_id UUID REFERENCES pockets(id);

-- Create default "Main" pocket for existing users
INSERT INTO pockets (user_id, name, is_default)
SELECT id, 'Main', TRUE FROM users;

-- Associate existing transactions with user's Main pocket
UPDATE transactions t 
SET pocket_id = p.id 
FROM pockets p 
WHERE t.user_id = p.user_id AND p.is_default = TRUE;

-- Make pocket_id NOT NULL after migration
ALTER TABLE transactions ALTER COLUMN pocket_id SET NOT NULL;
