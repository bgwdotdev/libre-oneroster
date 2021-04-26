INSERT
    OR IGNORE INTO scopes (
        scope)
    VALUES (
        'roster-core.readonly')
    , (
        'roster-core.createput')
    , (
        'admin.readonly');

INSERT
    OR IGNORE INTO StatusType (
        token)
    VALUES (
        'active')
    , (
        'tobedeleted');
