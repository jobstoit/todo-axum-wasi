-- Init (up)
CREATE TYPE priority AS ENUM (
	'none',
	'low',
	'medium',
	'high'
);

CREATE TABLE users (
	id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	username VARCHAR UNIQUE NOT NULL,
	password_hash VARCHAR NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE todos (
	id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	user_id UUID NOT NULL,
	title VARCHAR NOT NULL,
	priority priority NOT NULL DEFAULT 'none',
	description TEXT,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
	due_date TIMESTAMP,
	finished_at TIMESTAMP
);

ALTER TABLE todos
	ADD CONSTRAINT fk_todos_user_id
	FOREIGN KEY (user_id) REFERENCES users(id);

CREATE TABLE sessions (
	user_id UUID NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	valid_until TIMESTAMP NOT NULL DEFAULT NOW() + INTERVAL '3h',
	token VARCHAR NOT NULL UNIQUE
);

ALTER TABLE sessions
	ADD CONSTRAINT fk_sessions_user_id
	FOREIGN KEY (user_id) REFERENCES users(id);
