INSERT INTO
    roles (role_name)
VALUES
    ('Admin'),
    ('User')
ON CONFLICT DO NOTHING;

INSERT INTO
    users (user_name, email, password_hash, role_id)
SELECT
    'admin user',
    'm21137@g.metro-cit.ac.jp',
    '$2b$12$NgBBXLLqabBE3XD8K38Hd.nWdb9DYANGl/CfnJ/v6kptzRkimT1pe',
    role_id
FROM
    roles
WHERE
    role_name LIKE 'Admin';

INSERT INTO
    users (user_name, email, password_hash, role_id)
SELECT
    'common user',
    'horikawa0107tokyo@gmail.com',
    '$2b$12$NgBBXLLqabBE3XD8K38Hd.nWdb9DYANGl/CfnJ/v6kptzRkimT1pe',
    role_id
FROM
    roles
WHERE
    role_name LIKE 'User';

INSERT INTO
    spaces (user_id, description,is_active,space_name,capacity,equipment,address)
SELECT
     user_id,
    'test',
    true,
    'meeting room1',
    10,
    'projector',
    '東京都'
FROM
    users
WHERE
    user_name LIKE 'admin user';

