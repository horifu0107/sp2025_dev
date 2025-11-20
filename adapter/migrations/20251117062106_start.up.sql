-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS '
  BEGIN
    new.updated_at := ''now'';
    return new;
  END;
' LANGUAGE 'plpgsql';

CREATE TABLE IF NOT EXISTS spaces (
    space_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    space_name VARCHAR(255) NOT NULL,
    owner VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL,
    description VARCHAR(1024) NOT NULL,
    capacity INT NOT NULL,
    equipment VARCHAR(255) NOT NULL,
    address VARCHAR(255) NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
);

-- roles テーブルと users テーブルを追加する
CREATE TABLE IF NOT EXISTS roles (
  role_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  role_name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS users (
  user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL UNIQUE,
  password_hash VARCHAR(255) NOT NULL,
  role_id UUID NOT NULL ,
  created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  FOREIGN KEY (role_id) REFERENCES roles(role_id)
  ON UPDATE CASCADE
  ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS reservations (
    reservation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    space_id UUID NOT NULL,
    created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    reservation_start_time TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    reservation_end_time TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    FOREIGN KEY (user_id) REFERENCES users(user_id)
      ON UPDATE CASCADE
      ON DELETE CASCADE,
    FOREIGN KEY (space_id) REFERENCES spaces(space_id)
      ON UPDATE CASCADE
      ON DELETE CASCADE
);



CREATE TABLE IF NOT EXISTS reminders (
  reminder_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  is_already BOOLEAN NOT NULL,
  reservation_id UUID NOT NULL ,
  remind_time TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  FOREIGN KEY (reservation_id) REFERENCES reservations(reservation_id)
  ON UPDATE CASCADE
  ON DELETE CASCADE
);

CREATE TRIGGER spaces_updated_at_trigger
    BEFORE UPDATE ON spaces FOR EACH ROW
    EXECUTE PROCEDURE set_updated_at();