-- Create user_subscriptions table
CREATE TABLE user_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    pocket_id UUID REFERENCES pockets(id) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    amount DECIMAL(15,2) NOT NULL,
    basis VARCHAR(10) NOT NULL CHECK (basis IN ('monthly', 'annually')),
    billing_day INT NOT NULL CHECK (billing_day BETWEEN 1 AND 31),
    billing_month INT CHECK (billing_month BETWEEN 1 AND 12),
    category_id INT REFERENCES categories(id),
    is_active BOOLEAN DEFAULT TRUE,
    next_charge_date DATE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT annual_requires_month CHECK (basis != 'annually' OR billing_month IS NOT NULL)
);

-- Index for scheduler performance (find due subscriptions)
CREATE INDEX idx_user_subscriptions_scheduler 
ON user_subscriptions(is_active, next_charge_date) 
WHERE is_active = TRUE;

-- Add 'Subscriptions' category if it doesn't exist
INSERT INTO categories (name, is_income, icon, exclude_from_analysis)
SELECT 'Subscriptions', FALSE, 'subscriptions', FALSE
WHERE NOT EXISTS (SELECT 1 FROM categories WHERE name = 'Subscriptions');
