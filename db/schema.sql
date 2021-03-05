PRAGMA forgein_keys = 1;

CREATE TABLE IF NOT EXISTS credentials (
    id integer PRIMARY KEY AUTOINCREMENT
    , client_id text UNIQUE NOT NULL
    , client_secret text NOT NULL
    , tag text NOT NULL
);

CREATE TABLE IF NOT EXISTS scopes (
    id integer PRIMARY KEY AUTOINCREMENT
    , scope text UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS credential_scopes (
    id integer PRIMARY KEY AUTOINCREMENT
    , credential_id integer NOT NULL
    , scope_id integer NOT NULL
    , FOREIGN KEY (credential_id) REFERENCES credentials (id) ON DELETE CASCADE
    , FOREIGN KEY (scope_id) REFERENCES scopes (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS academicSessions (
    id integer PRIMARY KEY AUTOINCREMENT
    , sourcedId text UNIQUE NOT NULL
    , data json NOT NULL
);
