-- Add down migration script here

DROP VIEW IF EXISTS meals_view;

DROP INDEX IF EXISTS idx_meals_date_canteen_latest;

DROP INDEX IF EXISTS idx_meals_refreshed_at;

DELETE FROM meals WHERE is_latest = FALSE;

ALTER TABLE meals
DROP CONSTRAINT meals_pkey;

ALTER TABLE meals
DROP COLUMN id;

ALTER TABLE meals
ADD CONSTRAINT meals_pkey PRIMARY KEY (date, canteen, name);

ALTER TABLE meals
DROP COLUMN is_latest;

ALTER TABLE meals
DROP COLUMN refreshed_at;

ALTER TABLE meals
ALTER COLUMN dish_type
TYPE TEXT
USING dish_type::TEXT;

DROP TABLE IF EXISTS canteens_scraped;

DROP TYPE IF EXISTS dish_type_enum;