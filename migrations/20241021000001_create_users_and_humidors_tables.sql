-- Create users and humidors tables

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    is_setup_complete BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Humidors table
CREATE TABLE humidors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    description TEXT,
    capacity INTEGER,
    target_humidity INTEGER,
    location VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_humidors_user_id ON humidors(user_id);
CREATE INDEX idx_humidors_name ON humidors(name);

-- Add user_id to cigars table to associate cigars with users
ALTER TABLE cigars ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE cigars ADD COLUMN humidor_id UUID REFERENCES humidors(id) ON DELETE SET NULL;

-- Create indexes for the new foreign keys
CREATE INDEX idx_cigars_user_id ON cigars(user_id);
CREATE INDEX idx_cigars_humidor_id ON cigars(humidor_id);