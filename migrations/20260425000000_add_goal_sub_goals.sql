CREATE TABLE goal_sub_goals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    goal_id UUID NOT NULL REFERENCES financial_goals(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    target_amount DECIMAL(15,2) NOT NULL,
    position INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT goal_sub_goals_target_amount_positive CHECK (target_amount > 0),
    CONSTRAINT goal_sub_goals_position_non_negative CHECK (position >= 0)
);

CREATE UNIQUE INDEX idx_goal_sub_goals_goal_position ON goal_sub_goals(goal_id, position);
CREATE INDEX idx_goal_sub_goals_goal_id ON goal_sub_goals(goal_id);
