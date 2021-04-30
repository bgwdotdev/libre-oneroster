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
        'tobedeleted')
    , (
        'inactive');

INSERT
    OR IGNORE INTO OrgType (
        token)
    VALUES (
        'department')
    , (
        'school')
    , (
        'district')
    , (
        'local')
    , (
        'state')
    , (
        'national');

INSERT
    OR IGNORE INTO ClassType (
        token)
    VALUES (
        'homeroom')
    , (
        'scheduled');

INSERT
    OR IGNORE INTO RoleType (
        token)
    VALUES (
        'administrator')
    , (
        'aide')
    , (
        'guardian')
    , (
        'parent')
    , (
        'proctor')
    , (
        'relative')
    , (
        'student')
    , (
        'teacher');

INSERT
    OR IGNORE INTO SessionType (
        token)
    VALUES (
        'gradingPeriod')
    , (
        'semester')
    , (
        'schoolYear')
    , (
        'term');

INSERT OR IGNORE INTO GradeType (token, description) VALUES 
    ('IT', 'Infant/toddler'),
    ('PR', 'Preschool'),
    ('PK', 'Prekindergarten'),
    ('TK', 'Transitional Kindergarten'),
    ('KG', 'Kindergarten'),
    ('01', 'First grade'),
    ('02', 'Second grade'),
    ('03', 'Third grade'),
    ('04', 'Fourth grade'),
    ('05', 'Fifth grade'),
    ('06', 'Sixth grade'),
    ('07', 'Seventh grade'),
    ('08', 'Eigth grade'),
    ('09', 'Ninth grade'),
    ('10', 'Tenth grade'),
    ('11', 'Eleventh grade'),
    ('12', 'Twelfth grade'),
    ('13', 'Grade 13'),
    ('PS', 'Postsecondary'),
    ('UG', 'Ungraded'),
    ('Other', 'Other');
