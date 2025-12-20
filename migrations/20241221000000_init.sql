-- 1. Users Table (The Core)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Currencies (Reference Table)
CREATE TABLE currencies (
    code VARCHAR(3) PRIMARY KEY, -- "USD", "SGD", "IDR"
    symbol VARCHAR(5),           -- "$", "Rp"
    name VARCHAR(50)
);

-- Seed Currencies
INSERT INTO currencies (code, symbol, name) VALUES 
('SGD', 'S$', 'Singapore Dollar'),
('USD', '$', 'US Dollar'),
('IDR', 'Rp', 'Indonesian Rupiah');

-- 3. User Preferences (Which currency do they use?)
CREATE TABLE user_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    base_currency VARCHAR(3) REFERENCES currencies(code) DEFAULT 'SGD'
);

-- 4. Categories (Now Shared OR Private? Let's make them Global for simplicity first)
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    is_income BOOLEAN DEFAULT FALSE
);

-- 5. Transactions (Linked to User)
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL, -- <--- THE KEY CHANGE
    amount DECIMAL(19, 4) NOT NULL,
    description TEXT,
    category_id INT REFERENCES categories(id),
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 6. Assets (Global - Everyone sees the same "AAPL")
CREATE TABLE assets (
    ticker VARCHAR(10) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    asset_type VARCHAR(20) NOT NULL
);

-- 7. Portfolio (User Specific)
CREATE TABLE portfolio (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) NOT NULL, -- <--- THE KEY CHANGE
    ticker VARCHAR(10) REFERENCES assets(ticker),
    quantity DECIMAL(19, 8) NOT NULL,
    avg_buy_price DECIMAL(19, 4) NOT NULL,
    current_price DECIMAL(19, 4),
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, ticker) -- A user can't have two separate rows for AAPL
);

-- SEED DATA
-- Create a default user so you can test immediately
INSERT INTO users (id, username, email, password_hash) VALUES 
('00000000-0000-0000-0000-000000000001', 'jonathan', 'jonathan@example.com', '$argon2id$v=19$m=19456,t=2,p=1$wl6e559kWWZdrwN8I1CjJQ$Jk32W6Sjol4vvZt0pEUIjvwhjcqYCT8w1xrzBosg7nQ');

INSERT INTO user_settings (user_id, base_currency) VALUES
('00000000-0000-0000-0000-000000000001', 'SGD');

INSERT INTO categories (name, is_income) VALUES 
('Food', false), ('Transport', false), ('Salary', true);

INSERT INTO assets (ticker, name, asset_type) VALUES ('AAPL', 'Apple', 'Stock');