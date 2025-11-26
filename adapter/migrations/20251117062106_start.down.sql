-- Add down migration script here
DROP TABLE IF EXISTS reservations;
DROP TABLE IF EXISTS returned_reservations;
DROP TRIGGER IF EXISTS spaces_updated_at_trigger ON spaces;
DROP TABLE IF EXISTS spaces;

-- DROP TRIGGER IF EXISTS users_updated_at_trigger ON users;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS roles;

DROP FUNCTION set_updated_at;