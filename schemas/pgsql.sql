CREATE TABLE clients (
	id SERIAL PRIMARY KEY NOT NULL,
	display_name TEXT NOT NULL
);

CREATE TABLE client_api_keys (
  id SERIAL PRIMARY KEY NOT NULL,
  valid_to TIMESTAMP WITH TIME ZONE NOT NULL,
  key TEXT NOT NULL,
  client_id INTEGER NOT NULL,
  FOREIGN KEY(client_id) REFERENCES clients(id)
);

CREATE INDEX client_api_valid_to_index ON client_api_keys(client_id, valid_to);

CREATE TABLE work_lists (
  id SERIAL PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  client_id INTEGER NOT NULL,
  FOREIGN KEY(client_id) REFERENCES clients(id)
);

CREATE TABLE todos (
  id SERIAL PRIMARY KEY NOT NULL,
  content TEXT NOT NULL,
  completed BOOLEAN NOT NULL,
  work_list_id INTEGER NOT NULL,
  FOREIGN KEY(work_list_id) REFERENCES work_lists(id)
);
