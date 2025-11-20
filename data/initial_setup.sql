INSERT INTO
    roles (role_name)
VALUES
    ('Admin'),
    ('User')
ON CONFLICT DO NOTHING;

INSERT INTO
    users (user_name, email, password_hash, role_id)
SELECT
    'Eleazar Fig',
    'eleazar.fig@example.com',
    '$2b$12$NgBBXLLqabBE3XD8K38Hd.nWdb9DYANGl/CfnJ/v6kptzRkimT1pe',
    role_id
FROM
    roles
WHERE
    role_name LIKE 'Admin';