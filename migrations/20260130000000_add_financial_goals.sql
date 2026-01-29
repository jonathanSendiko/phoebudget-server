-- Create financial_goals table
CREATE TABLE financial_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    pocket_id UUID REFERENCES pockets(id) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    target_amount DECIMAL(15,2) NOT NULL,
    icon VARCHAR(50) DEFAULT 'savings',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for faster lookup by user
CREATE INDEX idx_financial_goals_user_id ON financial_goals(user_id);
