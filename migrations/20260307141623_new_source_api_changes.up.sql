-- Add up migration script here

ALTER TYPE dish_type_enum ADD VALUE IF NOT EXISTS 'soup' AFTER 'side';
ALTER TYPE dish_type_enum ADD VALUE IF NOT EXISTS 'other' AFTER 'dessert';


ALTER TABLE meals
ADD COLUMN saturated_fats NUMERIC(6,2);