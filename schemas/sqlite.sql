CREATE TABLE todos (
  id INTEGER PRIMARY KEY NOT NULL,
  content TEXT NOT NULL,
  completed BOOLEAN NOT NULL,
  work_list_id INTEGER NOT NULL,
  FOREIGN KEY(work_list_id) REFERENCES work_lists(id)
);

CREATE TABLE clients (
  id INTEGER PRIMARY KEY NOT NULL,
  display_name TEXT NOT NULL
);

CREATE TABLE client_api_keys (
  id INTEGER PRIMARY KEY NOT NULL,
  valid_to INTEGER NOT NULL,
  key TEXT NOT NULL,
  client_id INTEGER NOT NULL,
  FOREIGN KEY(client_id) REFERENCES clients(id)
);

CREATE INDEX client_api_valid_to_index ON client_api_keys(client_id, valid_to);

CREATE TABLE work_lists (
  id INTEGER PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  client_id INTEGER NOT NULL,
  FOREIGN KEY(client_id) REFERENCES clients(id)
);
