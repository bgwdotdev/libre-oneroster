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
    , dateLastModified text NOT NULL
    , name text NOT NULL
    , orgTypeId text NOT NULL
    , identifier text
    , parent text
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (orgTypeId) REFERENCES OrgType (id)
    , FOREIGN KEY (parent) REFERENCES orgs (sourcedId)
);

CREATE VIEW IF NOT EXISTS orgs_json AS
    SELECT json_object(
        'sourcedId', o.sourcedId
        , 'status', st.token
        , 'dateLastModified', o.dateLastModified
        , 'name', o.name
        , 'type', ot.token
        , 'identifier', o.identifier
        , 'parent', CASE WHEN o.parent IS NOT NULL THEN 
            json_object(
                'href', 'orgs/' || o.parent
                , 'sourcedId', o.parent
                , 'type', 'org'
            ) ELSE NULL 
        END
        , 'children', CASE WHEN op.sourcedId IS NOT NULL THEN
            json_group_array(
                json_object(
                    'href', 'orgs/' || op.sourcedId
                    , 'sourcedId', op.sourcedId
                    , 'type', 'org'
                ) 
            ) ELSE NULL 
        END
    ) AS 'org'
    FROM
        orgs o
        LEFT JOIN orgs op ON o.sourcedId = op.parent
        LEFT JOIN StatusType st ON o.statusTypeId = st.id
        LEFT JOIN OrgType ot on o.orgTypeId = ot.id
    GROUP BY
        o.sourcedId
    ORDER BY
        o.sourcedId
;

CREATE TABLE IF NOT EXISTS StatusType (
    id integer PRIMARY KEY AUTOINCREMENT
    , token text UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS OrgType (
    id integer PRIMARY KEY AUTOINCREMENT
    , token text UNIQUE NOT NULL
);
