-- Add up migration script here

ALTER TABLE meals
ADD COLUMN kjoules INT;

ALTER TABLE meals
ADD COLUMN proteins NUMERIC(6,2);

ALTER TABLE meals
ADD COLUMN carbohydrates NUMERIC(6,2);

ALTER TABLE meals
ADD COLUMN fats NUMERIC(6,2);

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