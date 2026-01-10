-- Add subscriptions table for premium tier tracking
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan VARCHAR(20) NOT NULL DEFAULT 'free', -- 'free', 'premium', 'lifetime'
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- 'active', 'cancelled', 'expired'
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- NULL for lifetime or free
    payment_provider VARCHAR(50), -- 'stripe', 'google_play', 'app_store', 'manual'
    external_subscription_id VARCHAR(255), -- Provider's subscription ID
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);

-- Create default free subscription for existing users
INSERT INTO subscriptions (user_id, plan, status)
SELECT id, 'free', 'active' FROM users
ON CONFLICT DO NOTHING;
