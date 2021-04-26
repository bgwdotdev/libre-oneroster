-- TODO: make table names UpperCamelCase
-- TODO: make columns camelCase ?
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

CREATE TABLE IF NOT EXISTS orgs (
    id text PRIMARY KEY
    , sourcedId text UNIQUE NOT NULL
    , statusTypeId integer NOT NULL
    , name text NOT NULL
    , parent text
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (parent) REFERENCES orgs (sourcedId)
);

CREATE VIEW IF NOT EXISTS orgs_json AS
    SELECT json_object(
        'sourcedId', o.sourcedId
        , 'status', s.token
        , 'name', o.name
        , 'parent', o.parent
        , 'children', json_group_array(oc.sourcedId)
    ) AS 'org'
    FROM
        orgs o
        LEFT JOIN orgs oc ON o.sourcedId = oc.parent
        LEFT JOIN StatusType s ON o.statusTypeId = s.id
    GROUP BY
        o.sourcedId
    ORDER BY
        o.sourcedId
;

CREATE TABLE IF NOT EXISTS StatusType (
    id integer PRIMARY KEY AUTOINCREMENT
    , token text UNIQUE NOT NULL
);
