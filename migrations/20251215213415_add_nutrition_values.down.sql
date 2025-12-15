-- Add down migration script here

DROP VIEW IF EXISTS meals_view;

ALTER TABLE meals
DROP COLUMN kjoules;

ALTER TABLE meals
DROP COLUMN proteins;

ALTER TABLE meals
DROP COLUMN carbohydrates;

ALTER TABLE meals
DROP COLUMN fats;