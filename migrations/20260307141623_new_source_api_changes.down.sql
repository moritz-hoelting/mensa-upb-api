-- Add down migration script here

ALTER TABLE meals
DROP COLUMN saturated_fats;

DROP VIEW IF EXISTS meals_view;

UPDATE meals
SET dish_type = 'main'
WHERE dish_type IN ('soup', 'other');

ALTER TYPE dish_type_enum RENAME TO dish_type_enum_old;

CREATE TYPE dish_type_enum AS ENUM ('main', 'side', 'dessert');

ALTER TABLE meals
  ALTER COLUMN dish_type
  TYPE dish_type_enum
  USING dish_type::text::dish_type_enum;

DROP TYPE dish_type_enum_old;

CREATE OR REPLACE VIEW meals_view AS
SELECT
    id,
    date,
    canteen,
    name,
    dish_type,
    image_src,
    price_students,
    price_employees,
    price_guests,
    vegan,
    vegetarian,
    kjoules,
    proteins,
    carbohydrates,
    fats,
    round(kjoules / 4.184) AS kcal
FROM meals
WHERE is_latest = TRUE;