-- Add deleted_at column to transactions table for soft delete support
ALTER TABLE transactions
ADD COLUMN deleted_at TIMESTAMPTZ;
