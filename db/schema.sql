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
    , "parentSourcedId" integer
    , "schoolYear" integer -- YYYY
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (sessionTypeId) REFERENCES SessionType (id)
    , FOREIGN KEY (parentSourcedId) REFERENCES AcademicSessions (sourcedId)
);

-- Custom
CREATE TABLE IF NOT EXISTS Subjects (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "title" text NOT NULL
    , "subjectCode" text NOT NULL
);

-- OR:4.3
CREATE TABLE IF NOT EXISTS Classes (
    "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "title" text NOT NULL
    , "classCode" text
    , "classTypeId" integer NOT NULL
    , "location" text
    , "courseSourcedId" integer NOT NULL
    , "schoolSourcedId" integer NOT NULL
    , "termsSourcedId" integer NOT NULL
    , "resourcesSourcedId" integer
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (classTypeId) REFERENCES ClassType (id) 
    , FOREIGN KEY (courseSourcedId) REFERENCES Courses (sourcedId)
    , FOREIGN KEY (schoolSourcedId) REFERENCES Orgs (sourcedId)
    , FOREIGN KEY (termsSourcedId) REFERENCES academicSessions (sourcedId)
    , FOREIGN KEY (resourcesSourcedId) REFERENCES Resources (sourcedId)
);

CREATE TABLE IF NOT EXISTS ClassGrades (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "classSourcedId" integer NOT NULL
    , "gradeTypeId" integer NOT NULL
    , FOREIGN KEY (classSourcedId) REFERENCES Classes (sourcedId)
    , FOREIGN KEY (gradeTypeId) REFERENCES GradeType (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS ClassGradeIndex ON ClassGrades (classSourcedId, gradeTypeId);

CREATE TABLE IF NOT EXISTS ClassSubjects (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "classSourcedId" integer NOT NULL
    , "subjectSourcedId" integer NOT NULL
    , FOREIGN KEY (classSourcedId) REFERENCES Classes (sourcedId)
    , FOREIGN KEY (subjectSourcedId) REFERENCES Subjects (sourcedId)
);
CREATE UNIQUE INDEX IF NOT EXISTS ClassSubjectIndex ON ClassSubjects (classSourcedId, subjectSourcedId);

CREATE TABLE IF NOT EXISTS Periods (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "sourcedId" text UNIQUE NOT NULL
    , "title" text NOT NULL
    , "periodCode" text NOT NULL
    , "description" text
    , "orgSourcedId" integer NOT NULL
    , FOREIGN KEY (orgSourcedId) REFERENCES Orgs (sourcedId)
);
-- TODO: required?
CREATE UNIQUE INDEX IF NOT EXISTS OrgPeriods ON Periods (orgSourcedId, sourcedId);

CREATE TABLE IF NOT EXISTS ClassPeriods (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "classSourcedId" integer NOT NULL
    , "periodSourcedId" integer NOT NULL
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
    , "schoolYearSourcedId" integer
    , "courseCode" text
    , "orgSourcedId" integer NOT NULL
    , "resourcesSourcedId" integer
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (schoolYearSourcedId) REFERENCES AcademicSessions (sourcedId)
    , FOREIGN KEY (orgSourcedId) REFERENCES Orgs (sourcedId)
    , FOREIGN KEY (resourcesSourcedId) REFERENCES Resources (sourcedId)
);

CREATE TABLE IF NOT EXISTS CourseGrades (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "courseSourcedId" integer NOT NULL
    , "gradeTypeId" integer NOT NULL
    , FOREIGN KEY (courseSourcedId) REFERENCES Courses (sourcedId)
    , FOREIGN KEY (gradeTypeId) REFERENCES GradeType (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS CourseGradeIndex ON CourseGrades (courseSourcedId, gradeTypeId);

CREATE TABLE IF NOT EXISTS CourseSubjects (
    "id" integer NOT NULL
    , "courseSourcedId" integer NOT NULL
    , "subjectSourcedId" integer NOT NULL
    , FOREIGN KEY (courseSourcedId) REFERENCES Courses (sourcedId)
    , FOREIGN KEY (subjectSourcedId) REFERENCES Subjects (sourcedId)
);
CREATE UNIQUE INDEX IF NOT EXISTS CourseSubjectIndex ON CourseSubjects (courseSourcedId, subjectSourcedId);

-- Demographics not supported

-- OR:4.6
CREATE TABLE IF NOT EXISTS Enrollments (
    "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "userSourcedId" integer NOT NULL
    , "classSourcedId" integer NOT NULL
    , "SchoolSourcedId" integer NOT NULL
    , "roleTypeId" integer NOT NULL
    , "primary" integer -- bool 0/1
    , "beginDate" text
    , "endDate" text
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId)
    , FOREIGN KEY (classSourcedId) REFERENCES Classes (sourcedId)
    , FOREIGN KEY (schoolSourcedId) REFERENCES Orgs (sourcedId)
    , FOREIGN KEY (roleTypeId) REFERENCES RoleType (id)
);

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
    , FOREIGN KEY (parentSourcedId) REFERENCES orgs (sourcedId)
);

-- OR:4.12
CREATE TABLE IF NOT EXISTS Users (
    "sourcedId" text UNIQUE NOT NULL
    , "statusTypeId" integer NOT NULL
    , "dateLastModified" text NOT NULL
    , "username" text NOT NULL
    , "enabledUser" integer NOT NULL -- bool
    , "givenName" text NOT NULL
    , "familyName" text NOT NULL
    , "middleName" text
    , "roleTypeId" integer NOT NULL
    , "identifier" text
    , "email" text
    , "sms" text
    , "phone" text
    , "agentsSourcedId" integer
    , "orgsSourcedId" integer NOT NULL
    , "password" text
    , FOREIGN KEY (statusTypeId) REFERENCES StatusType (id)
    , FOREIGN KEY (roleTypeId) REFERENCES RoleType (id)
    , FOREIGN KEY (agentsSourcedId) REFERENCES Users (sourcedId)
    , FOREIGN KEY (orgsSourcedId) REFERENCES Orgs (sourcedId)
);

CREATE TABLE IF NOT EXISTS UserIds (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "userSourcedId" integer NOT NULL
    , "type" text NOT NULL
    , "identifier" text NOT NULL
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId)
);

CREATE TABLE IF NOT EXISTS UserGrades (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "userSourcedId" integer NOT NULL
    , "gradeTypeId" integer NOT NULL
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId)
    , FOREIGN KEY (gradeTypeId) REFERENCES GradeType (id)
);
CREATE UNIQUE INDEX IF NOT EXISTS UserGradesIndex ON UserGrades (userSourcedId, gradeTypeId);

CREATE TABLE IF NOT EXISTS UserAgents (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "userSourcedId" integer NOT NULL
    , "agentUserSourcedId" integer NOT NULL
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId)
    , FOREIGN KEY (agentUserSourcedId) REFERENCES Users (sourcedId)
);
CREATE UNIQUE INDEX IF NOT EXISTS UserAgentsIndex ON UserAgents (userSourcedId, agentUserSourcedId);

CREATE TABLE IF NOT EXISTS UserOrgs (
    "id" integer PRIMARY KEY AUTOINCREMENT
    , "userSourcedId" integer NOT NULL
    , "orgSourcedId" integer NOT NULL
    , FOREIGN KEY (userSourcedId) REFERENCES Users (sourcedId)
    , FOREIGN KEY (orgSourcedId) REFERENCES Orgs (sourcedId)
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

-- OR 5.2
CREATE VIEW IF NOT EXISTS ClassesJson AS
    SELECT json_object(
        'sourcedId', c.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', c.dateLastModified
        , 'title', c.title
        , 'classCode', c.classCode
        , 'classType', ClassType.token
        , 'location', c.location
        , 'grades', json_group_array(GradeType.token)
        , 'subjects', json_group_array(Subjects.title)
        , 'course', json_object(
            'href', 'courses/' || c.courseSourcedId
            , 'sourcedId', c.courseSourcedId
            , 'type', 'course'
        ) 
        , 'school', json_object(
            'href', 'orgs/' || c.schoolSourcedId
            , 'sourcedId', c.schoolSourcedId
            , 'type', 'org'
        )
        , 'terms', json_object(
            'href', 'academicSessions/' || c.termsSourcedId
            , 'sourcedId', c.termsSourcedId
            , 'type', 'academicSession'
        )
        , 'subjectCodes', json_group_array(Subjects.subjectCode)
        , 'periods', json_group_array(Periods.periodCode)
        -- TODO: resources
    ) AS 'class'
    FROM
        Classes c
        LEFT JOIN StatusType ON c.statusTypeId = StatusType.id
        LEFT JOIN ClassType ON c.classTypeId = ClassType.id
        LEFT JOIN ClassGrades ON c.sourcedId = ClassGrades.classSourcedId
        LEFT JOIN GradeType ON ClassGrades.gradeTypeId = GradeType.id
        LEFT JOIN ClassSubjects ON c.sourcedId = ClassSubjects.classSourcedId
        LEFT JOIN Subjects ON ClassSubjects.subjectSourcedId = Subject.sourcedId
        LEFT JOIN ClassPeriods ON c.sourcedId = ClassPeriods.classSourcedId
        LEFT JOIN Periods ON ClassPeriods.periodSourcedId = Periods.sourcedId
    GROUP BY
        c.sourcedId
    ORDER BY
        c.sourcedId
;

-- OR 5.3
CREATE VIEW IF NOT EXISTS CoursesJson AS
    SELECT json_object(
        'sourcedId', Courses.soucedId
        , 'status', StatusType.token
        , 'dateLastModified', Courses.dateLastModified
        , 'title', Courses.title
        , 'schoolYear', CASE WHEN Courses.schoolYear IS NOT NULL THEN
            json_group_array( json_object (
                    'href', 'academicSessions/' || Courses.schoolYearSourcedId
                    , 'sourcedId', Courses.schoolYearSourcedId
                    , 'type', 'academicSession'
            )) ELSE NULL END
        , 'courseCode', Courses.courseCode
        , 'grades', json_group_array(GradeType.token)
        , 'subjects', json_group_array(Subjects.title)
        , 'org', json_object(
            'href', 'orgs/' || Courses.orgSourcedId
            , 'sourcedId', Courses.orgSourcedId
            , 'type', 'org'
        )
        , 'subjectCodes', json_group_array(Subject.subjectCode)
        -- TODO: resources
    ) AS 'course' 
    FROM
        Courses
        LEFT JOIN StatusType ON Courses.statusTypeId = StatusType.id
        LEFT JOIN CourseGrades ON Courses.sourcedId = CourseGrades.courseSourcedId
        LEFT JOIN GradeType ON CourseGrades.gradeTypeId = GradeType.id
        LEFT JOIN CourseSubjects ON Courses.sourcedId = CourseSubjects.courseSourcedId
        LEFT JOIN Subjects ON CourseSubjects.subjectSourcedId = Subject.id
    GROUP BY
        Course.sourcedId
    ORDER BY
        Course.sourcedId
;

-- OR 5.5
CREATE VIEW IF NOT EXISTS EnrollmentsJson AS
    SELECT json_object(
        'sourcedId', Enrollments.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Enrollments.dateLastModified
        , 'role', RoleType.token
        , 'primary', Enrollments."primary"
        , 'user', json_object(
            'href', 'users' || Enrollments.userSourcedId
            , 'sourcedId', Enrollments.userSourcedId
            , 'type', 'user'
        )
        , 'class', json_object(
            'href', 'classes' || Enrollments.classSourcedId
            , 'sourcedId', Enrollments.classSourcedId
            , 'type', 'class'
        )
        , 'school', json_object(
            'href', 'orgs' || Enrollments.schoolSourcedId
            , 'sourcedId', Enrollments.schoolSourcedId
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

-- OR 5.11
CREATE VIEW IF NOT EXISTS UsersJson AS
    SELECT json_object(
        'sourcedId', Users.sourcedId
        , 'status', StatusType.token
        , 'dateLastModified', Users.dateLastModified
        , 'username', Users.username
        , 'UserIds', CASE WHEN UserIds.userSourcedId IS NOT NULL THEN
            json_group_array( json_object(
                'type', UserIds."type"
                , 'identifier', UserIds.identifier
            )) ELSE NULL END
        , 'enabledUser', Users.enabledUser
        , 'givenName', Users.givenName
        , 'familyName', Users.familyName
        , 'middleName', Users.middleName
        , 'role', RoleType.token
        , 'identifier', Users.identifier
        , 'email', Users.email
        , 'sms', Users.sms
        , 'phone', Users.phone
        , 'agents', CASE WHEN UserAgents.userSourcedId IS NOT NULL THEN
            json_group_array( json_object(
                    'href', 'users/' || UserAgents.agentUserSourcedId
                    , 'sourcedId', UserAgents.agentUserSourcedId
                    , 'type', 'user'
            )) ELSE NULL END
        , 'orgs', CASE WHEN UserOrgs.userSourcedId IS NOT NULL THEN
            json_group_array( json_object(
                    'href', 'orgs/' || UserOrgs.orgSourcedId
                    , 'sourcedId', UserOrgs.orgSourcedId
                    , 'type', 'org'
            )) ELSE NULL END
        , 'grades', json_group_array(GradeType.token)
        , 'password', Users.password
    ) AS 'user'
    FROM
        Users
        LEFT JOIN StatusType ON Users.statusTypeId = StatusType.id
        LEFT JOIN UserIds ON Users.sourcedId = UserIds.userSourcedId
        LEFT JOIN RoleType ON Users.roleTypeId = RoleType.id
        LEFT JOIN UserAgents ON Users.sourcedId = UserAgents.userSourcedId
        LEFT JOIN UserOrgs ON Users.sourcedId = UserOrgs.userSourcedId
        LEFT JOIN UserGrades ON Users.sourcedId = UserGrades.userSourcedId
        LEFT JOIN GradeType ON UserGrades.gradeTypeId = GradeType.id
    GROUP BY
        Users.sourcedId
    ORDER BY
        Users.sourcedId
;
