-- Add down migration script here

ALTER TABLE meals ALTER COLUMN vegan DROP NOT NULL;
ALTER TABLE meals ALTER COLUMN vegetarian DROP NOT NULL;