ALTER TABLE goal_entries
    ADD COLUMN sub_goal_id UUID REFERENCES goal_sub_goals(id) ON DELETE SET NULL;

CREATE INDEX idx_goal_entries_sub_goal_id ON goal_entries(sub_goal_id);
