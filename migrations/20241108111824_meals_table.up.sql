-- Add up migration script here

CREATE TABLE IF NOT EXISTS meals(
    date DATE NOT NULL,
    canteen TEXT NOT NULL,
    name TEXT NOT NULL,
    dish_type TEXT NOT NULL,
    image_src TEXT,
    price_students DECIMAL(5, 2) NOT NULL,
    price_employees DECIMAL(5, 2) NOT NULL,
    price_guests DECIMAL(5, 2) NOT NULL,
    vegan BOOLEAN DEFAULT FALSE,
    vegetarian BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (date, canteen, name)
);