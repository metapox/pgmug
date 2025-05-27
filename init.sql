-- Initialize test database with sample data

-- Create a users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample data
INSERT INTO users (name, email) VALUES 
    ('Alice Johnson', 'alice@example.com'),
    ('Bob Smith', 'bob@example.com'),
    ('Charlie Brown', 'charlie@example.com'),
    ('Diana Prince', 'diana@example.com'),
    ('Edward Norton', 'edward@example.com');

-- Create a products table for more complex queries
CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    price DECIMAL(10,2) NOT NULL,
    category VARCHAR(100),
    in_stock BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample products
INSERT INTO products (name, price, category, in_stock) VALUES 
    ('Laptop Computer', 999.99, 'Electronics', true),
    ('Coffee Mug', 12.50, 'Kitchen', true),
    ('Office Chair', 299.00, 'Furniture', true),
    ('Smartphone', 699.99, 'Electronics', false),
    ('Desk Lamp', 45.99, 'Furniture', true);

-- Create orders table
CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    product_id INTEGER REFERENCES products(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert sample orders
INSERT INTO orders (user_id, product_id, quantity) VALUES 
    (1, 1, 1),
    (2, 2, 2),
    (3, 3, 1),
    (1, 5, 1),
    (4, 1, 1);
