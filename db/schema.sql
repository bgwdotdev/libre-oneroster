-- TODO: make table names UpperCamelCase
-- TODO: make columns camelCase ?
PRAGMA forgein_keys = 1;

-- Auth tables

CREATE TABLE IF NOT EXISTS credentials (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "client_id" text UNIQUE NOT NULL
    , "client_secret" text NOT NULL
    , "tag" text NOT NULL
);

CREATE TABLE IF NOT EXISTS scopes (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "scope" text UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS credential_scopes (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "credential_id" integer NOT NULL
    , "scope_id" integer NOT NULL
    , FOREIGN KEY (credential_id) REFERENCES credentials (id) ON DELETE CASCADE
    , FOREIGN KEY (scope_id) REFERENCES scopes (id) ON DELETE CASCADE
);

-- OR:4

-- OR:4.2
CREATE TABLE IF NOT EXISTS AcademicSessions (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "title" text NOT NULL
    , "startDate" text NOT NULL
    , "endDate" text NOT NULL
    , "sessionTypeId" integer NOT NULL
    , "parentSourcedId" text
    , "schoolYear" text -- YYYY
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (sessionTypeId) REFERENCES SessionType (id)
    , FOREIGN KEY (parentSourcedId) REFERENCES AcademicSessions (sourcedId) DEFERRABLE INITIALLY DEFERRED
);

-- Custom
CREATE TABLE IF NOT EXISTS Subjects (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "title" text NOT NULL
    , "subjectCode" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
);

-- OR:4.3
CREATE TABLE IF NOT EXISTS Classes (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "title" text NOT NULL
    , "classCode" text
    , "classTypeId" integer NOT NULL
    , "location" text
    , "courseSourcedId" text NOT NULL
    , "orgSourcedId" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (classTypeId) REFERENCES ClassType (id)
    , FOREIGN KEY (courseSourcedId) REFERENCES Courses (sourcedId)
    , FOREIGN KEY (orgSourcedId) REFERENCES Orgs (sourcedId)
);

CREATE TABLE IF NOT EXISTS ClassGrades (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "classSourcedId" text NOT NULL
    , "gradeTypeId" integer NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (classSourcedId) REFERENCES Classes (sourcedId)
    , FOREIGN KEY (gradeTypeId) REFERENCES GradeType (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS ClassGradeIndex ON ClassGrades (classSourcedId, gradeTypeId);

CREATE TABLE IF NOT EXISTS ClassSubjects (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "classSourcedId" text NOT NULL
    , "subjectSourcedId" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (classSourcedId) REFERENCES Classes (sourcedId)
    , FOREIGN KEY (subjectSourcedId) REFERENCES Subjects (sourcedId)
);
CREATE UNIQUE INDEX IF NOT EXISTS ClassSubjectIndex ON ClassSubjects (classSourcedId, subjectSourcedId);

CREATE TABLE IF NOT EXISTS ClassAcademicSessions (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "classSourcedId" text NOT NULL
    , "academicSessionSourcedId" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (classSourcedId) REFERENCES Classes (sourcedId)
    , FOREIGN KEY (academicSessionSourcedId) REFERENCES AcademicSessions (sourcedId)
);
CREATE UNIQUE INDEX IF NOT EXISTS ClassAcademicSessionsIndex ON ClassAcademicSessions (classSourcedId, academicSessionSourcedId);

CREATE TABLE IF NOT EXISTS Periods (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "title" text NOT NULL
    , "periodCode" text NOT NULL
    , "description" text
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
);

CREATE TABLE IF NOT EXISTS OrgPeriods (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "periodSourcedId" text NOT NULL
    , "orgSourcedId" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (orgSourcedId) REFERENCES Orgs (sourcedId)
    , FOREIGN KEY (periodSourcedId) REFERENCES Periods (sourcedId)
);
CREATE UNIQUE INDEX IF NOT EXISTS OrgPeriodsIndex ON OrgPeriods (periodSourcedId, orgSourcedId);

CREATE TABLE IF NOT EXISTS ClassPeriods (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "classSourcedId" text NOT NULL
    , "periodSourcedId" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (classSourcedId) REFERENCES Classes (sourcedId)
    , FOREIGN KEY (periodSourcedId) REFERENCES Periods (sourcedId)
);
CREATE UNIQUE INDEX IF NOT EXISTS ClassPeriodIndex ON ClassPeriods (classSourcedId, periodSourcedId);

-- OR:4.4
CREATE TABLE IF NOT EXISTS Courses (
    "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "title" text NOT NULL
    , "schoolYearSourcedId" text
    , "courseCode" text
    , "orgSourcedId" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (schoolYearSourcedId) REFERENCES AcademicSessions (sourcedId)
    , FOREIGN KEY (orgSourcedId) REFERENCES Orgs (sourcedId)
);

CREATE TABLE IF NOT EXISTS CourseGrades (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "courseSourcedId" text NOT NULL
    , "gradeTypeId" integer NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (courseSourcedId) REFERENCES Courses (sourcedId)
    , FOREIGN KEY (gradeTypeId) REFERENCES GradeType (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS CourseGradeIndex ON CourseGrades (courseSourcedId, gradeTypeId);

CREATE TABLE IF NOT EXISTS CourseSubjects (
    "id" integer NOT NULL
    , "statusTypeId" integer NOT NULL
    , "courseSourcedId" text NOT NULL
    , "subjectSourcedId" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (courseSourcedId) REFERENCES Courses (sourcedId)
    , FOREIGN KEY (subjectSourcedId) REFERENCES Subjects (sourcedId)
);
CREATE UNIQUE INDEX IF NOT EXISTS CourseSubjectIndex ON CourseSubjects (courseSourcedId, subjectSourcedId);

-- Demographics not supported

-- OR:4.6
CREATE TABLE IF NOT EXISTS Enrollments (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "userSourcedId" text NOT NULL
    , "classSourcedId" text NOT NULL
    , "orgSourcedId" text NOT NULL
    , "roleTypeId" integer NOT NULL
    , "primary" integer -- bool 0/1
    , "beginDate" text
    , "endDate" text
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId)
    , FOREIGN KEY (classSourcedId) REFERENCES Classes (sourcedId)
    , FOREIGN KEY (orgSourcedId) REFERENCES Orgs (sourcedId)
    , FOREIGN KEY (roleTypeId) REFERENCES RoleType (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS EnrollmentsIndex ON Enrollments (userSourcedId, classSourcedId, orgSourcedId);

-- OR:4.9
CREATE TABLE IF NOT EXISTS Orgs (
    "id" text PRIMARY KEY
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "name" text NOT NULL
    , "orgTypeId" text NOT NULL
    , "identifier" text
    , "parentSourcedId" text
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (orgTypeId) REFERENCES OrgType (id)
    , FOREIGN KEY (parentSourcedId) REFERENCES orgs (sourcedId) DEFERRABLE INITIALLY DEFERRED
);

-- OR:4.12
CREATE TABLE IF NOT EXISTS Users (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "username" text NOT NULL
    , "enabledUser" boolean NOT NULL -- bool
    , "givenName" text NOT NULL
    , "familyName" text NOT NULL
    , "middleName" text
    , "roleTypeId" integer NOT NULL
    , "identifier" text
    , "email" text
    , "sms" text
    , "phone" text
    , "password" text
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id) ON DELETE RESTRICT
    , FOREIGN KEY (roleTypeId) REFERENCES RoleType (id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS UserIds (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "userSourcedId" text NOT NULL
    , "type" text NOT NULL
    , "identifier" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id) ON DELETE RESTRICT
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS UserIdsIndex ON UserIds ("userSourcedId", "type");

CREATE TABLE IF NOT EXISTS UserGrades (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "userSourcedId" text NOT NULL
    , "gradeTypeId" integer NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id) ON DELETE RESTRICT
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId) ON DELETE CASCADE
    , FOREIGN KEY (gradeTypeId) REFERENCES GradeType (id) ON DELETE RESTRICT
);
CREATE UNIQUE INDEX IF NOT EXISTS UserGradesIndex ON UserGrades (userSourcedId, gradeTypeId);

CREATE TABLE IF NOT EXISTS UserAgents (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "userSourcedId" text NOT NULL
    , "agentUserSourcedId" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id) ON DELETE RESTRICT
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED
    , FOREIGN KEY (agentUserSourcedId) REFERENCES Users (sourcedId) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED
);
CREATE UNIQUE INDEX IF NOT EXISTS UserAgentsIndex ON UserAgents (userSourcedId, agentUserSourcedId);

CREATE TABLE IF NOT EXISTS UserOrgs (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "statusTypeId" integer NOT NULL
    , "userSourcedId" text NOT NULL
    , "orgSourcedId" text NOT NULL
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id) ON DELETE RESTRICT
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId) ON DELETE CASCADE
    , FOREIGN KEY (orgSourcedId) REFERENCES Orgs (sourcedId) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS UserOrgsIndex ON UserOrgs (userSourcedId, orgSourcedId);

/* TODO:

Line Items
Line Items Categories
Resources
Results

*/

-- OR:4.13

CREATE TABLE IF NOT EXISTS ClassType (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "token" text UNIQUE NOT NULL
);

-- Gender unsupported

-- TODO: ImportanceType

CREATE TABLE IF NOT EXISTS OrgType (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "token" text UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS RoleType (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "token" text UNIQUE NOT NULL
);

-- TODO: ScoreStatus

CREATE TABLE IF NOT EXISTS SessionType (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "token" text UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS StatusType (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "token" text UNIQUE NOT NULL
);

-- CEDS v5 Entry Grade Level: https://ceds.ed.gov/CEDSElementDetails.aspx?TermId=7100
CREATE TABLE IF NOT EXISTS GradeType (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "token" text UNIQUE NOT NULL
    , "description" text NOT NULL
);

-- OR:5.1
CREATE VIEW IF NOT EXISTS AcademicSessionsJsonArray AS
    SELECT json_object(
        'academicSessions', json_group_array(json(academicSession))
    ) AS 'academicSessions'
FROM AcademicSessionsJson
;

CREATE VIEW IF NOT EXISTS AcademicSessionsJson AS
    SELECT json_object(
        'sourcedId', a.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', a.dateLastModified
        , 'title', a.title
        , 'startDate', a.startDate
        , 'endDate', a.endDate
        , 'type', SessionType.token
        , 'parent', CASE WHEN a.parentSourcedId IS NOT NULL THEN
            json_object(
                'href', 'academicSessions/' || a.parentSourcedId
                , 'sourcedId', a.parentSourcedId
                , 'type', 'academicSession'
            ) ELSE NULL
        END
        , 'children', CASE WHEN ap.sourcedId IS NOT NULL THEN
            json_group_array(
                json_object(
                    'href', 'academicSessions/' || ap.sourcedId
                    , 'sourcedId', ap.sourcedId
                    , 'type', 'academicSession'
                )
            ) ELSE NULL
        END
        , 'schoolYear', a.schoolYear
    ) AS 'academicSession'
    FROM
        AcademicSessions a
        LEFT JOIN AcademicSessions ap ON a.sourcedId = ap.parentSourcedId
        LEFT JOIN StatusType ON a.statusTypeId = StatusType.id
        LEFT JOIN SessionType ON a.sessionTypeId = SessionType.id
    GROUP BY
        a.sourcedId
    ORDER BY
        a.sourcedId
;

CREATE VIEW IF NOT EXISTS PeriodsJsonArray AS
    SELECT json_object(
        'periods', json_group_array(json(period))
    ) AS 'periods'
FROM PeriodsJson
;

CREATE VIEW IF NOT EXISTS PeriodsJson AS
    SELECT json_object(
        'sourcedId', Periods.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Periods.dateLastModified
        , 'title', Periods.title
        , 'periodCode', Periods.periodCode
        , 'description', Periods.description
        , 'orgs', json(OP.orgs)
    ) AS 'period'
    FROM
        Periods
        LEFT JOIN StatusType ON Periods.statusTypeId = StatusType.id
        LEFT JOIN (
            SELECT
                periodSourcedId
                , json_group_array(json_object(
                    'href', 'users/' || OrgPeriods.orgSourcedId
                    , 'sourcedId', OrgPeriods.orgSourcedId
                    , 'type', 'org'
                )) AS orgs
            FROM OrgPeriods
            WHERE statusTypeId = ( SELECT id FROM StatusType WHERE token = 'active' )
            GROUP BY periodSourcedId
        ) AS OP ON Periods.sourcedId = OP.periodSourcedId
    GROUP BY
        Periods.sourcedId
    ORDER BY
        Periods.sourcedId
;

CREATE VIEW IF NOT EXISTS SubjectsJsonArray AS
    SELECT json_object(
        'subjects', json_group_array(json(subject))
    ) AS 'subjects'
FROM SubjectsJson
;

CREATE VIEW IF NOT EXISTS SubjectsJson AS
    SELECT json_object(
        'sourcedId', Subjects.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Subjects.dateLastModified
        , 'title', Subjects.title
        , 'subjectCode', Subjects.subjectCode
    ) AS 'subject'
    FROM
        Subjects
        LEFT JOIN StatusType ON Subjects.statusTypeId = StatusType.id
    ORDER BY
        Subjects.sourcedId
;

-- OR 5.3
CREATE VIEW IF NOT EXISTS CoursesJsonArray AS
    SELECT json_object(
        'courses', json_group_array(json(course))
    ) AS 'courses'
FROM CoursesJson
;

CREATE VIEW IF NOT EXISTS CoursesJson AS
    SELECT json_object(
        'sourcedId', Courses.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Courses.dateLastModified
        , 'title', Courses.title
        , 'schoolYear', CASE WHEN Courses.schoolYearSourcedId IS NOT NULL THEN
            json_object (
                    'href', 'academicSessions/' || Courses.schoolYearSourcedId
                    , 'sourcedId', Courses.schoolYearSourcedId
                    , 'type', 'academicSession'
            ) ELSE NULL END
        , 'courseCode', Courses.courseCode
        , 'grades', CASE WHEN CourseGrades.courseSourcedId IS NOT NULL THEN
            json_group_array(GradeType.token)
        ELSE NULL END
        , 'subjects', CASE WHEN CourseSubjects.courseSourcedId IS NOT NULL THEN
            json_group_array(Subjects.title)
        ELSE NULL END
        , 'org', json_object(
            'href', 'orgs/' || Courses.orgSourcedId
            , 'sourcedId', Courses.orgSourcedId
            , 'type', 'org'
        )
        , 'subjectCodes', CASE WHEN CourseSubjects.courseSourcedId IS NOT NULL THEN
            json_group_array(Subjects.subjectCode)
        ELSE NULL END
        -- TODO: resources
    ) AS 'course'
    FROM
        Courses
        LEFT JOIN StatusType ON Courses.statusTypeId = StatusType.id
        LEFT JOIN CourseGrades ON Courses.sourcedId = CourseGrades.courseSourcedId
        LEFT JOIN GradeType ON CourseGrades.gradeTypeId = GradeType.id
        LEFT JOIN CourseSubjects ON Courses.sourcedId = CourseSubjects.courseSourcedId
        LEFT JOIN Subjects ON CourseSubjects.subjectSourcedId = Subjects.id
    GROUP BY
        Courses.sourcedId
    ORDER BY
        Courses.sourcedId
;

-- OR 5.5
CREATE VIEW IF NOT EXISTS EnrollmentsJsonArray AS
    SELECT json_object(
        'enrollments', json_group_array(json(enrollment))
    ) AS 'enrollments'
FROM EnrollmentsJson
;

CREATE VIEW IF NOT EXISTS EnrollmentsJson AS
    SELECT json_object(
        'sourcedId', Enrollments.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Enrollments.dateLastModified
        , 'role', RoleType.token
        , 'primary', Enrollments."primary"
        , 'user', json_object(
            'href', 'users/' || Enrollments.userSourcedId
            , 'sourcedId', Enrollments.userSourcedId
            , 'type', 'user'
        )
        , 'class', json_object(
            'href', 'classes/' || Enrollments.classSourcedId
            , 'sourcedId', Enrollments.classSourcedId
            , 'type', 'class'
        )
        , 'school', json_object(
            'href', 'orgs/' || Enrollments.orgSourcedId
            , 'sourcedId', Enrollments.orgSourcedId
            , 'type', 'org'
        )
        , 'beginDate', Enrollments.beginDate
        , 'endDate', Enrollments.endDate
    ) AS 'enrollment'
    FROM
        Enrollments
        LEFT JOIN StatusType ON Enrollments.statusTypeId = StatusType.id
        LEFT JOIN RoleType ON Enrollments.roleTypeId = RoleType.id
    GROUP BY
        Enrollments.sourcedId
    ORDER BY
        Enrollments.sourcedId
;

-- TODO: update styling
-- OR:5.8
CREATE VIEW IF NOT EXISTS OrgsJsonArray AS
    SELECT json_object(
        'orgs', json_group_array(json(org))
    ) AS 'orgs'
FROM OrgsJson
;

CREATE VIEW IF NOT EXISTS OrgsJson AS
    SELECT json_object(
        'sourcedId', Orgs.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Orgs.dateLastModified
        , 'name', Orgs.name
        , 'type', OrgType.token
        , 'identifier', Orgs.identifier
        , 'parent', CASE WHEN Orgs.parentSourcedId IS NOT NULL THEN
            json_object(
                'href', 'orgs/' || Orgs.parentSourcedId
                , 'sourcedId', Orgs.parentSourcedId
                , 'type', 'org'
            ) ELSE NULL
        END
        , 'children', CASE WHEN OrgParent.sourcedId IS NOT NULL THEN
            json_group_array(
                json_object(
                    'href', 'orgs/' || OrgParent.sourcedId
                    , 'sourcedId', OrgParent.sourcedId
                    , 'type', 'org'
                )
            ) ELSE NULL
        END
    ) AS 'org'
    FROM
        Orgs
        LEFT JOIN Orgs OrgParent ON Orgs.sourcedId = OrgParent.parentSourcedId
        LEFT JOIN StatusType ON Orgs.statusTypeId = StatusType.id
        LEFT JOIN OrgType ON Orgs.orgTypeId = OrgType.id
    GROUP BY
        Orgs.sourcedId
    ORDER BY
        Orgs.sourcedId
;

CREATE VIEW IF NOT EXISTS ClassesJsonArray AS
    SELECT json_object('classes', json_group_array(json(class))) AS 'classes' FROM ClassesJson;

CREATE VIEW IF NOT EXISTS ClassesJson AS
    SELECT json_object(
        'sourcedId', Classes.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Classes.dateLastModified
        , 'title', Classes.title
        , 'classCode', Classes.classCode
        , 'classType', ClassType.token
        , 'locations', Classes.location
        , 'grades', json(CG.grades)
        , 'subjects', json(CS.title)
        , 'course', json_object(
            'href', 'courses/' || Classes.courseSourcedId
            , 'sourcedId', Classes.courseSourcedId
            , 'type', 'course'
        )
        , 'school', json_object(
            'href', 'orgs/' || Classes.orgSourcedId
            , 'sourcedId', Classes.orgSourcedId
            , 'type', 'org'
        )
        , 'terms', json(CA.terms)
        , 'subjectCodes', json(CS.code)
        , 'periods', json(CP.period)
    ) AS 'class'
    FROM
        Classes
        LEFT JOIN StatusType ON Classes.statusTypeId = StatusType.id
        LEFT JOIN ClassType ON Classes.classTypeId = ClassType.id
        LEFT JOIN (
            SELECT
                classSourcedId
                , json_group_array(
                    GradeType.token
                ) AS grades
            FROM ClassGrades
            LEFT JOIN GradeType ON ClassGrades.gradeTypeId = GradeType.id
            WHERE statusTypeId = ( SELECT id FROM StatusType WHERE token = 'active' )
            GROUP BY classSourcedId
        ) AS CG ON Classes.sourcedId = CG.classSourcedId
        LEFT JOIN (
            SELECT
                classSourcedId
                , Subjects.sourcedId -- needed to avoid proc macro bug?
                , json_group_array(
                    Subjects.title
                ) AS title
                , json_group_array(
                    Subjects.subjectCode
                ) AS code
            FROM ClassSubjects
            LEFT JOIN Subjects ON ClassSubjects.subjectSourcedId = Subjects.sourcedId
            WHERE ClassSubjects.statusTypeId = ( SELECT id FROM StatusType WHERE token = 'active' )
            GROUP BY classSourcedId
        ) AS CS ON Classes.sourcedId = CS.classSourcedId
        LEFT JOIN (
            SELECT
                classSourcedId
                , json_group_array(json_object(
                        'href', 'academicSessions/' || ClassAcademicSessions.academicSessionSourcedId
                        , 'sourcedId', ClassAcademicSessions.academicSessionSourcedId
                        , 'type', 'academicSession'
                )) AS terms
            FROM ClassAcademicSessions
            WHERE statusTypeId = ( SELECT id FROM StatusType WHERE token = 'active' )
            GROUP BY classSourcedId
        ) AS CA ON Classes.sourcedId = CA.classSourcedId
        LEFT JOIN (
            SELECT
                classSourcedId
                , Periods.sourcedId
                , json_group_array(
                    Periods.periodCode
                ) AS period
            FROM ClassPeriods
            LEFT JOIN Periods ON ClassPeriods.periodSourcedId = Periods.sourcedId
            WHERE ClassPeriods.statusTypeId = ( SELECT id FROM StatusType WHERE token = 'active' )
            GROUP BY classSourcedId
        ) AS CP ON Classes.sourcedId = CP.classSourcedId
    GROUP BY
        Classes.sourcedId
    ORDER BY
        Classes.sourcedId
;

CREATE VIEW IF NOT EXISTS VwORGetClass AS
    SELECT json_object(
        'class', json(class)
    ) AS 'class'
FROM ClassesJson
;

-- OR 5.11
CREATE VIEW IF NOT EXISTS UsersJsonArray AS
    SELECT json_object(
        'users', json_group_array(json("user"))
    ) AS 'users'
FROM UsersJson
;

CREATE VIEW IF NOT EXISTS UsersJson AS
    SELECT json_object(
        'sourcedId', Users.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Users.dateLastModified
        , 'username', Users.username
        , 'userIds', json(UI.userIds)
        , 'enabledUser', Users.enabledUser
        , 'givenName', Users.givenName
        , 'familyName', Users.familyName
        , 'middleName', Users.middleName
        , 'role', RoleType.token
        , 'identifier', Users.identifier
        , 'email', Users.email
        , 'sms', Users.sms
        , 'phone', Users.phone
        , 'agents', json(UA.agents)
        , 'orgs', json(UO.orgs)
        , 'grades', CASE WHEN UserGrades.userSourcedId IS NOT NULL THEN
            json_group_array(GradeType.token)
        ELSE NULL END
        , 'password', Users.password
    ) AS 'user'
    FROM
        Users
        LEFT JOIN StatusType ON Users.statusTypeId = StatusType.id
        LEFT JOIN (
            SELECT
                userSourcedId
                , json_group_array(json_object(
                    'type', "type"
                    , 'identifier', identifier
                )) AS userIds
            FROM UserIds
            WHERE statusTypeId = ( SELECT id FROM StatusType WHERE token = 'active' )
            GROUP BY userSourcedId
        ) AS UI ON Users.sourcedId = UI.userSourcedId
        LEFT JOIN RoleType ON Users.roleTypeId = RoleType.id
        LEFT JOIN (
            SELECT
                userSourcedId
                , json_group_array(json_object(
                    'href', 'users/' || UserAgents.agentUserSourcedId
                    , 'sourcedId', UserAgents.agentUserSourcedId
                    , 'type', 'user'
                )) AS agents
            FROM UserAgents
            WHERE statusTypeId = ( SELECT id FROM StatusType WHERE token = 'active' )
            GROUP BY userSourcedId
        ) AS UA ON Users.sourcedId = UA.userSourcedId
        LEFT JOIN (
            SELECT
                userSourcedId
                , json_group_array(json_object(
                    'href', 'orgs/' || UserOrgs.orgSourcedId
                    , 'sourcedId', UserOrgs.orgSourcedId
                    , 'type', 'org'
                )) AS orgs
            FROM UserOrgs
            WHERE statusTypeId = ( SELECT id FROM StatusType WHERE token = 'active' )
            GROUP BY userSourcedId
        ) AS UO ON Users.SourcedId = UO.userSourcedId
        LEFT JOIN UserGrades ON Users.sourcedId = UserGrades.userSourcedId
            AND UserGrades.statusTypeId = ( SELECT id FROM StatusType WHERE token = 'active' )
        LEFT JOIN GradeType ON UserGrades.gradeTypeId = GradeType.id
    GROUP BY
        Users.sourcedId
    ORDER BY
        Users.sourcedId
;

CREATE TRIGGER IF NOT EXISTS TriggerUpsertAcademicSessionsJson
    INSTEAD OF INSERT ON AcademicSessionsJson
    FOR EACH ROW
BEGIN
    INSERT INTO AcademicSessions (sourcedId
        , statusTypeId
        , dateLastModified
        , title
        , startDate
        , endDate
        , sessionTypeId
        , parentSourcedId
        , schoolYear
    )
    VALUES (
        json_extract(NEW.academicSession, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = json_extract(NEW.academicSession, '$.status'))
        , strftime('%Y-%m-%dT%H:%M:%fZ', json_extract(NEW.academicSession, '$.dateLastModified'))
        , json_extract(NEW.academicSession, '$.title')
        , date(json_extract(NEW.academicSession, '$.startDate'))
        , date(json_extract(NEW.academicSession, '$.endDate'))
        , (SELECT id FROM SessionType WHERE token = json_extract(NEW.academicSession, '$.type'))
        , json_extract(NEW.academicSession, '$.parent.sourcedId')
        , json_extract(NEW.academicSession, '$.schoolYear')
    )
    ON CONFLICT (sourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
        , dateLastModified=excluded.dateLastModified
        , title=excluded.title
        , startDate=excluded.startDate
        , endDate=excluded.endDate
        , sessionTypeId=excluded.sessionTypeId
        , parentSourcedId=excluded.parentSourcedId
        , schoolYear=excluded.schoolYear
    ;
END;

CREATE TRIGGER IF NOT EXISTS TriggerUpsertPeriodsJson
    INSTEAD OF INSERT ON PeriodsJson
    FOR EACH ROW
BEGIN
    INSERT INTO Periods (
        sourcedId
        , statusTypeId
        , dateLastModified
        , title
        , periodCode
        , description
    )
    VALUES (
        json_extract(NEW.period, '$.sourcedId')
        , ( SELECT id FROM StatusType WHERE token = json_extract(NEW.period, '$.status'))
        , strftime('%Y-%m-%dT%H:%M:%fZ', json_extract(NEW.period, '$.dateLastModified'))
        , json_extract(NEW.period, '$.title')
        , json_extract(NEW.period, '$.periodCode')
        , json_extract(NEW.period, '$.description')
    )
    ON CONFLICT (sourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
        , dateLastModified=excluded.dateLastModified
        , title=excluded.title
        , periodCode=excluded.periodCode
        , description=excluded.description
    ;

    UPDATE OrgPeriods
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE periodSourcedId = json_extract(NEW.period, '$.sourcedId');

    INSERT OR IGNORE INTO OrgPeriods(
        periodSourcedId
        , statusTypeId
        , orgSourcedId
    )
    SELECT
        json_extract(NEW.period, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , json_extract(org.value, '$.sourcedId')
    FROM
        json_each(NEW.period, '$.orgs') AS  org
    WHERE true
    ON CONFLICT (periodSourcedId, orgSourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;
END;

CREATE TRIGGER IF NOT EXISTS TriggerUpsertClassesJson
    INSTEAD OF INSERT ON ClassesJson
    FOR EACH ROW
BEGIN
    INSERT INTO Classes (
        sourcedId
        , statusTypeId
        , dateLastModified
        , title
        , classCode
        , classTypeId
        , location
        , courseSourcedId
        , orgSourcedId
    )
    VALUES (
        json_extract(NEW.class, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = json_extract(NEW.class, '$.status'))
        , strftime('%Y-%m-%dT%H:%M:%fZ', json_extract(NEW.class, '$.dateLastModified'))
        , json_extract(NEW.class, '$.title')
        , json_extract(NEW.class, '$.classCode')
        , (SELECT id FROM classType WHERE token = json_extract(NEW.class, '$.classType'))
        , json_extract(NEW.class, '$.location')
        , json_extract(NEW.class, '$.course.sourcedId')
        , json_extract(NEW.class, '$.school.sourcedId')
    )
    ON CONFLICT (sourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
        , dateLastModified=excluded.dateLastModified
        , title=excluded.title
        , classCode=excluded.classCode
        , classTypeId=excluded.classTypeId
        , location=excluded.location
        , courseSourcedId=excluded.courseSourcedId
        , orgSourcedId=excluded.orgSourcedId
    ;

    UPDATE ClassGrades
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE classSourcedId = json_extract(NEW.class, '$.sourcedId');

    INSERT OR IGNORE INTO ClassGrades(
        classSourcedId
        , statusTypeId
        , gradeTypeId
    )
    SELECT
        json_extract(NEW.class, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , (SELECT id FROM GradeType WHERE token = grades.value)
    FROM
        json_each(NEW.class, '$.grades') AS grades
    WHERE true
    ON CONFLICT (classSourcedId, gradeTypeId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;

    UPDATE ClassSubjects
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE classSourcedId = json_extract(NEW.class, '$.sourcedId');

    INSERT OR IGNORE INTO ClassSubjects(
        classSourcedId
        , statusTypeId
        , subjectSourcedId
    )
    SELECT
        json_extract(NEW.class, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , (SELECT sourcedId FROM Subjects WHERE subjectCode = sc.value)
    FROM
        json_each(NEW.class, '$.subjectCodes') AS sc
    WHERE true
    ON CONFLICT (classSourcedId, subjectSourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;

    UPDATE ClassAcademicSessions
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE classSourcedId = json_extract(NEW.class, '$.sourcedId');

    INSERT OR IGNORE INTO ClassAcademicSessions(
        classSourcedId
        , statusTypeId
        , academicSessionSourcedId
    )
    SELECT
        json_extract(NEW.class, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , json_extract(term.value, '$.sourcedId')
    FROM
        json_each(NEW.class, '$.terms') AS term
    WHERE true
    ON CONFLICT (classSourcedId, academicSessionSourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;

    UPDATE ClassPeriods
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE classSourcedId = json_extract(NEW.class, '$.sourcedId');

    INSERT INTO ClassPeriods(
        classSourcedId
        , statusTypeId
        , periodSourcedId
    )
    SELECT
        json_extract(NEW.class, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , (SELECT sourcedId FROM Periods WHERE periodCode = pc.value)
    FROM
        json_each(NEW.class, '$.periods') AS pc
    WHERE true
    ON CONFLICT (classSourcedId, periodSourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;
END;

CREATE TRIGGER IF NOT EXISTS TriggerUpsertOrgsJson
    INSTEAD OF INSERT ON OrgsJson
    FOR EACH ROW
BEGIN
    INSERT INTO Orgs (sourcedId
    , statusTypeId
    , dateLastModified
    , name
    , orgTypeId
    , identifier
    , parentSourcedId
    )
    VALUES (
        json_extract(NEW.org, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = json_extract(NEW.org, '$.status'))
        , strftime('%Y-%m-%dT%H:%M:%fZ', json_extract(NEW.org, '$.dateLastModified'))
        , json_extract(NEW.org, '$.name')
        , ( SELECT id FROM OrgType WHERE token = json_extract(NEW.org, '$.type') )
        , json_extract(NEW.org, '$.identifier')
        , json_extract(NEW.org, '$.parent.sourcedId')
    )
    ON CONFLICT (sourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
        , dateLastModified=excluded.dateLastModified
        , name=excluded.name
        , orgTypeId=excluded.orgTypeId
        , identifier=excluded.identifier
        , parentSourcedId=excluded.parentSourcedId
    ;
END;

CREATE TRIGGER IF NOT EXISTS TriggerUpsertSubjectsJson
    INSTEAD OF INSERT ON SubjectsJson
    FOR EACH ROW
BEGIN
    INSERT INTO Subjects(
        sourcedId
        , statusTypeId
        , dateLastModified
        , title
        , subjectCode
    )
    VALUES(
        json_extract(NEW.subject, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = json_extract(NEW.subject, '$.status'))
        , strftime('%Y-%m-%dT%H:%M:%fZ', json_extract(NEW.subject, '$.dateLastModified'))
        , json_extract(NEW.subject, '$.title')
        , json_extract(NEW.subject, '$.subjectCode')
    )
    ON CONFLICT (sourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
        , dateLastModified=excluded.dateLastModified
        , title=excluded.title
        , subjectCode=excluded.subjectCode
    ;
END;

CREATE TRIGGER IF NOT EXISTS TriggerUpsertCoursesJson
    INSTEAD OF INSERT ON CoursesJson
    FOR EACH ROW
BEGIN
    INSERT INTO Courses(
        sourcedId
        , statusTypeId
        , dateLastModified
        , title
        , schoolYearSourcedId
        , courseCode
        , orgSourcedId
    )
    VALUES(
        json_extract(NEW.course, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = json_extract(NEW.course, '$.status'))
        , strftime('%Y-%m-%dT%H:%M:%fZ', json_extract(NEW.course, '$.dateLastModified'))
        , json_extract(NEW.course, '$.title')
        , json_extract(NEW.course, '$.schoolYear.sourcedId')
        , json_extract(NEW.course, '$.courseCode')
        , json_extract(NEW.course, '$.org.sourcedId')
    )
    ON CONFLICT (sourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
        , dateLastModified=excluded.dateLastModified
        , title=excluded.title
        , schoolYearSourcedId=excluded.schoolYearSourcedId
        , courseCode=excluded.courseCode
        , orgSourcedId=excluded.orgSourcedId
    ;

    UPDATE CourseGrades
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE courseSourcedId = json_extract(NEW.course, '$.sourcedId');

    INSERT OR IGNORE INTO CourseGrades(
        courseSourcedId
        , statusTypeId
        , gradeTypeId
    )
    SELECT
        json_extract(NEW.course, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , (SELECT id FROM GradeType WHERE token = grades.value)
    FROM
        json_each(NEW.course, '$.grades') AS grades
    WHERE true
    ON CONFLICT (courseSourcedId, gradeTypeId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;

    UPDATE CourseSubjects
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE courseSourcedId = json_extract(NEW.course, '$.sourcedId');

    INSERT OR IGNORE INTO CourseSubjects(
        courseSourcedId
        , statusTypeId
        , subjectSourcedId
    )
    SELECT
        json_extract(NEW.course, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , (SELECT sourcedId FROM Subjects WHERE subjectCode = sc.value)
    FROM
        json_each(NEW.course, '$.subjectCodes') AS sc
    WHERE true
    ON CONFLICT (courseSourcedId, subjectSourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;
END;

CREATE TRIGGER IF NOT EXISTS TriggerUpsertEnrollmentsJson
    INSTEAD OF INSERT ON EnrollmentsJson
    FOR EACH ROW
BEGIN
    INSERT INTO Enrollments (
        sourcedId
        , statusTypeId
        , dateLastModified
        , userSourcedId
        , classSourcedId
        , orgSourcedId
        , roleTypeId
        , "primary"
        , beginDate
        , endDate
    )
    VALUES (
        json_extract(NEW.enrollment, '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = json_extract(NEW.enrollment, '$.status'))
        , strftime('%Y-%m-%dT%H:%M:%fZ', json_extract(NEW.enrollment, '$.dateLastModified'))
        , json_extract(NEW.enrollment, '$.user.sourcedId')
        , json_extract(NEW.enrollment, '$.class.sourcedId')
        , json_extract(NEW.enrollment, '$.school.sourcedId')
        , (SELECT id FROM RoleType WHERE token = json_extract(NEW.enrollment, '$.role'))
        , json_extract(NEW.enrollment, '$.primary')
        , date(json_extract(NEW.enrollment, '$.beginDate'))
        , date(json_extract(NEW.enrollment, '$.endDate'))
    )
    ON CONFLICT (sourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
        , dateLastModified=excluded.dateLastModified
        , userSourcedId=excluded.userSourcedId
        , classSourcedId=excluded.classSourcedId
        , orgSourcedId=excluded.orgSourcedId
        , roleTypeId=excluded.roleTypeId
        , "primary"=excluded."primary"
        , beginDate=excluded.beginDate
        , endDate=excluded.endDate
    ;
END;

CREATE TRIGGER IF NOT EXISTS TriggerUpsertUsersJson
    INSTEAD OF INSERT ON UsersJson
    FOR EACH ROW
BEGIN
    INSERT INTO Users (sourcedId
        , statusTypeId
        , dateLastModified
        , username
        , enabledUser
        , givenName
        , familyName
        , middleName
        , roleTypeId
        , identifier
        , email
        , sms
        , phone
        , password
    )
    VALUES (
        json_extract(NEW."user", '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = json_extract(NEW."user", '$.status'))
        , strftime('%Y-%m-%dT%H:%M:%fZ', json_extract(NEW."user", '$.dateLastModified'))
        , json_extract(NEW."user", '$.username')
        , json_extract(NEW."user", '$.enabledUser')
        , json_extract(NEW."user", '$.givenName')
        , json_extract(NEW."user", '$.familyName')
        , json_extract(NEW."user", '$.middleName')
        , (SELECT id FROM RoleType WHERE token = json_extract(NEW."user", '$.role'))
        , json_extract(NEW."user", '$.identifier')
        , json_extract(NEW."user", '$.email')
        , json_extract(NEW."user", '$.sms')
        , json_extract(NEW."user", '$.phone')
        , json_extract(NEW."user", '$.password')
    )
    ON CONFLICT (sourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
        , dateLastModified=excluded.dateLastModified
        , username=excluded.username
        , enabledUser=excluded.enabledUser
        , givenName=excluded.givenName
        , familyName=excluded.familyName
        , middleName=excluded.middleName
        , roleTypeId=excluded.roleTypeId
        , identifier=excluded.identifier
        , email=excluded.email
        , sms=excluded.sms
        , phone=excluded.phone
        , password=excluded.password
    ;

    -- Upserts UserIds table
    UPDATE UserIds
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE userSourcedId = json_extract(NEW."user", '$.sourcedId');

    INSERT OR IGNORE INTO UserIds (userSourcedId
        , statusTypeId
        , "type"
        , identifier
    )
    SELECT
        json_extract(NEW."user", '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , json_extract(userIds.value, '$.type')
        , json_extract(userIds.value, '$.identifier')
    FROM
        json_each(NEW."user", '$.userIds') AS userIds
    WHERE true
    ON CONFLICT (userSourcedId, "type") DO UPDATE SET
        statusTypeId=excluded.statusTypeId
        , identifier=excluded.identifier
    ;

    -- Upserts UserOrgs table
    UPDATE UserOrgs
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE userSourcedId = json_extract(NEW."user", '$.sourcedId');

    INSERT OR IGNORE INTO UserOrgs(
        userSourcedId
        , statusTypeId
        , orgSourcedId
    )
    SELECT
        json_extract(NEW."user", '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , json_extract(orgs.value, '$.sourcedId')
    FROM
        json_each(NEW."user", '$.orgs') AS orgs
    WHERE true
    ON CONFLICT (userSourcedId, orgSourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;

    /*

       Upserts the UserAgents table

       This first sets a users User/Agent links to the 'tobedeleted' status
       then upserts the passed items to 'active' status.
       This avoids the need for an explicit delete command and instead assumes any
       entities not passed are now obsolete.
       Sets users User/Agent links to 'tobedeleted' status

    */
    UPDATE UserAgents
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE userSourcedId = json_extract(NEW."user", '$.sourcedId');

    INSERT OR IGNORE INTO UserAgents(
        userSourcedId
        , statusTypeId
        , agentUserSourcedId
    )
    SELECT
        json_extract(NEW."user", '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , json_extract(agents.value, '$.sourcedId')
    FROM
        json_each(NEW."user", '$.agents') AS agents
    WHERE true
    ON CONFLICT (userSourcedId, agentUserSourcedId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;

    -- Upserts UserGrades table
    UPDATE UserGrades
    SET statusTypeId = ( SELECT id FROM StatusType WHERE token = 'tobedeleted' )
    WHERE userSourcedId = json_extract(NEW."user", '$.sourcedId');

    INSERT OR IGNORE INTO UserGrades(
        userSourcedId
        , statusTypeId
        , gradeTypeId
    )
    SELECT
        json_extract(NEW."user", '$.sourcedId')
        , (SELECT id FROM StatusType WHERE token = 'active')
        , (SELECT id FROM GradeType WHERE token = grades.value)
    FROM
        json_each(NEW."user", '$.grades') AS grades
    WHERE true
    ON CONFLICT (userSourcedId, gradeTypeId) DO UPDATE SET
        statusTypeId=excluded.statusTypeId
    ;

END;
