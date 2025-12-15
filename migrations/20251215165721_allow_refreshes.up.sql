-- Add up migration script here

CREATE TABLE canteens_scraped (
    canteen TEXT NOT NULL,
    scraped_for DATE NOT NULL,
    scraped_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (canteen, scraped_for, scraped_at)
);

ALTER TABLE meals
ADD COLUMN id UUID NOT NULL DEFAULT gen_random_uuid();

-- Remove existing primary key constraints
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN
        SELECT conname
        FROM pg_constraint
        WHERE contype = 'p'
          AND conrelid = 'meals'::regclass
    LOOP
        EXECUTE format('ALTER TABLE meals DROP CONSTRAINT %I', r.conname);
    END LOOP;
END $$;

ALTER TABLE meals
ADD CONSTRAINT meals_pkey PRIMARY KEY (id);

ALTER TABLE meals
ADD COLUMN is_latest BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE meals
ADD COLUMN refreshed_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE TYPE dish_type_enum AS ENUM ('main', 'side', 'dessert');

ALTER TABLE meals
ALTER COLUMN dish_type
TYPE dish_type_enum
USING dish_type::dish_type_enum;

CREATE INDEX idx_meals_date_canteen_latest ON meals(date, canteen, is_latest);

CREATE INDEX idx_meals_refreshed_at ON meals(refreshed_at);

CREATE VIEW meals_view AS
SELECT id, date, canteen, name, dish_type, image_src, price_students, price_employees, price_guests, vegan, vegetarian
FROM meals
WHERE is_latest = TRUE;