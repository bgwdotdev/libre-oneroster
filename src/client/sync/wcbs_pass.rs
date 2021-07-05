use crate::{client, model};
use async_std::net::TcpStream;
use std::fmt;
use tiberius::{Client, SqlBrowser};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Surf(surf::Error),
    Tiberius(tiberius::error::Error),
    Json(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Surf(ref e) => e.fmt(f),
            Error::Tiberius(ref e) => e.fmt(f),
            Error::Json(ref e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            //Error::Surf(ref e) => Some(e),
            Error::Tiberius(ref e) => Some(e),
            Error::Json(ref e) => Some(e),
            _ => None,
        }
    }
}
macro_rules! into_error {
    ($from:ty, $to:expr) => {
        impl From<$from> for Error {
            fn from(err: $from) -> Error {
                $to(err)
            }
        }
    };
}

into_error!(surf::Error, Error::Surf);
into_error!(tiberius::error::Error, Error::Tiberius);
into_error!(serde_json::Error, Error::Json);

pub struct Config {
    pub database_ado_string: String,
    pub oneroster: client::Config,
    pub delta: String,
    pub academic_year: usize,
}

async fn connect_database(connection_string: &str) -> Client<TcpStream> {
    let creds = tiberius::Config::from_ado_string(connection_string).unwrap();
    log::debug!("SQL server connection info: {:?}", creds);
    let tcp = TcpStream::connect_named(&creds).await.unwrap();
    let client = Client::connect(creds, tcp).await.unwrap();
    return client;
}

struct SyncConf {
    database: tiberius::Client<TcpStream>,
    oneroster: surf::Client,
    token: String,
    delta: String,
    year: String,
}

async fn sync2<T>(config: &mut SyncConf, endpoint: &str, query: &str) -> Result<()>
where
    for<'a> T: serde::Deserialize<'a>,
    T: serde::Serialize,
{
    log::info!("Syncing {}...", endpoint);
    let rows = config
        .database
        .query(query, &[&config.delta, &config.year])
        .await?
        .into_first_result()
        .await?;
    for row in rows {
        if let Some(data) = row.try_get::<&str, _>(endpoint)? {
            let out: T = serde_json::from_str(&data)?;
            client::put_all(&config.oneroster, &config.token, out, endpoint).await?;
        }
    }
    Ok(())
}
pub async fn sync(config: Config) -> Result<()> {
    log::info!("seeking database...");

    //connect database
    let database = connect_database(&config.database_ado_string).await;

    //TODO: server return 403
    //connect oneroster
    let (oneroster, token) = client::connect(config.oneroster).await?;
    let delta = config.delta;
    let year = config.academic_year.to_string();

    let mut sync_conf = SyncConf {
        database,
        oneroster,
        token,
        delta,
        year,
    };

    sync2::<model::AcademicSessions>(&mut sync_conf, "academicSessions", QUERY_ACADEMIC_SESSIONS)
        .await?;
    sync2::<model::Orgs>(&mut sync_conf, "orgs", QUERY_ORGS).await?;
    sync2::<model::Subjects>(&mut sync_conf, "subjects", QUERY_SUBJECTS).await?;
    sync2::<model::Periods>(&mut sync_conf, "periods", QUERY_PERIODS).await?;
    sync2::<model::Courses>(&mut sync_conf, "courses", QUERY_COURSES).await?;
    sync2::<model::Classes>(&mut sync_conf, "classes", QUERY_CLASSES).await?;
    //TODO: flag for init? use temptable on insert?
    sync2::<model::Users>(&mut sync_conf, "users", QUERY_USERS).await?;
    sync2::<model::Enrollments>(&mut sync_conf, "enrollments", QUERY_ENROLLMENTS).await?;

    Ok(())
}

static QUERY_ACADEMIC_SESSIONS: &str = r#"
-- name: select-academicSession-years
declare @results nvarchar(max);
SET @results = (
    SELECT
        cast(year.year_id as varchar(36)) AS sourcedId
        , CASE WHEN year.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(year.last_amend_date AS datetimeoffset) AS dateLastModified
        , year.description AS title
        , cast(school_calendar.year_start AS date) AS startDate
        , cast(school_calendar.year_end AS date) AS endDate
        , 'schoolYear' AS 'type'
        , cast(year.code AS varchar(4)) AS schoolYear
    FROM dbo.year
        INNER JOIN dbo.school_calendar ON school_calendar.academic_year = year.code
    WHERE year.last_amend_date > @p1
        AND year.code > @p2
    ORDER BY sourcedId
    FOR JSON PATH, root('academicSessions')
)
SELECT @results AS academicSessions
"#;

static QUERY_ORGS: &str = r#"
-- name: select-orgs
declare @results nvarchar(max);
SET @results = (
    SELECT
        cast(school_id AS varchar(36)) AS sourcedId
        , CASE WHEN in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(last_amend_date AS datetimeoffset) AS dateLastModified
        , code AS name
        , 'school' AS 'type'
        , description AS identifier
        , NULL AS parent -- GUIRef[0..1]
        , NULL AS children -- GUIDRef[0..*]
    FROM dbo.school
    WHERE last_amend_date > @p1
    ORDER BY sourcedId
    FOR JSON PATH, root('orgs')
)
SELECT @results AS orgs;
"#;

static QUERY_SUBJECTS: &str = r#"
declare @result NVARCHAR(max);
SET @result = (
    SELECT
        cast(subject_id AS varchar(36)) AS sourcedId
        , CASE WHEN in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(last_amend_date AS datetimeoffset) AS dateLastModified
        , description AS title
        , code AS subjectCode
    FROM dbo.subject
    WHERE last_amend_date > @p1
    ORDER BY sourcedId
    FOR JSON PATH, root('subjects')
)
SELECT @result AS subjects
"#;

static QUERY_PERIODS: &str = r#"
declare @timetableid int = '43348527'; -- CHANGEME: Your current timetable ID
declare @results nvarchar(max);
SET @results = (
    SELECT
        cast(period_id AS varchar(36)) AS sourcedId
        , CASE WHEN in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(getdate() as datetimeoffset) AS dateLastModified
        , description AS title
        , concat('D', DAY_NUMBER, 'P', period_num) AS periodCode
        , (
            SELECT cast(school.school_id AS varchar(36)) AS sourcedId
            FROM dbo.school WHERE school.code = time_period.school FOR JSON PATH
        ) AS 'orgs'
    FROM dbo.time_period
    WHERE time_table_id = @timetableid
    FOR JSON PATH, root('periods')
)
SELECT @results AS 'periods'
"#;

static QUERY_COURSES: &str = r#"
declare @results nvarchar(max);
SET @results = (
    SELECT
        cast(subject.subject_id AS varchar(36)) AS sourcedId
        , CASE WHEN subject.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(subject.last_amend_date AS datetimeoffset) AS dateLastModified
        , subject.description AS title
        , NULL AS schoolYear
        , subject.code AS courseCode
        , NULL AS grades -- string[0..*]
        , NULL AS subjects
        , cast(school.school_id AS varchar(36)) AS 'org.sourcedId'
        , NULL AS subjectCodes
    FROM dbo.subject
        INNER JOIN dbo.school ON school.code = subject.school
    WHERE subject.last_amend_date > @p1
    ORDER BY sourcedId
    FOR JSON PATH, root('courses')
)
SELECT @results AS 'courses'
"#;

static QUERY_CLASSES: &str = r#"
-- name: select-classes-scheduled
SELECT cast((
    SELECT
        cast(subject_set.subject_set_id AS varchar(36)) AS sourcedId
        , CASE WHEN subject_set.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(subject_set.last_amend_date AS datetimeoffset) AS dateLastModified
        , subject_set.description AS title
        , subject_set.set_code AS classCode
        , 'scheduled' AS classType
        , NULL AS location -- subject_set.room
        , NULL AS grades
        , NULL AS subjects
        , cast(subject.subject_id AS varchar(36)) AS 'course.sourcedId'
        , cast(school.school_id AS varchar(36)) AS 'school.sourcedId'
        , (
            SELECT cast(year.year_id AS varchar(36)) AS sourcedId
            FROM dbo.year WHERE year.code = subject_set.academic_year FOR JSON PATH
        ) AS terms
        , NULL AS subjectCodes
        , NULL AS periods
    FROM dbo.subject_set
        INNER JOIN dbo.school ON school.code = subject_set.school
        INNER JOIN dbo.subject ON subject.code = subject_set.subject
    WHERE subject_set.last_amend_date > @p1
        AND subject_set.academic_year = @p2
    ORDER BY sourcedId
    FOR JSON PATH, root('classes')
    ) AS nvarchar(max)
) AS classes

-- name: select-classes-homeroom
UNION
SELECT cast((
    SELECT
        cast(form.form_id AS varchar(36)) AS sourcedId
        , CASE WHEN form.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(form.last_amend_date AS datetimeoffset) AS dateLastModified
        , form.description AS title
        , form.code AS classCode
        , 'homeroom' AS classType
        , form.room AS location
        , NULL AS grades --form_year.age_range AS grades -- array
        , NULL AS subjects
        , cast((SELECT subject_id FROM subject WHERE description = 'Tutor Group') AS varchar(36)) AS 'course.sourcedId'
        --'40705670' AS 'course.sourcedId' -- what's this again? 'tutorial'? CHANGE FOR YOUR IMPORT
        , cast(school.school_id AS varchar(36)) AS 'school.sourcedId'
        , (
            SELECT cast(year.year_id AS varchar(36)) AS sourcedId
            FROM dbo.year WHERE year.code = form.academic_year FOR JSON PATH
        ) AS terms
        , NULL AS subjectCodes
        , NULL AS periods
    FROM dbo.form
        INNER JOIN dbo.school ON school.code = form.school
        INNER JOIN dbo.form_year ON form_year.code = form.year_code
    WHERE form.last_amend_date > @p1
        AND form.academic_year = @p2
    ORDER BY sourcedId
    FOR JSON PATH, root('classes')
    ) AS nvarchar(max)
)
"#;

static QUERY_USERS: &str = r#"
-- name: select-users-pupil
SELECT cast((
    SELECT
        cast(pupil.name_id AS varchar(36)) AS sourcedId
        , CASE WHEN pupil.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(pupil.last_amend_date AS datetimeoffset) AS dateLastModified
        , CASE WHEN name.email_address IS NULL THEN pupil.code ELSE name.email_address END AS username
        , NULL AS userIds
        , CASE WHEN pupil.in_use = 'Y' THEN 1 ELSE 0 END AS enabledUser -- change to PASS API 'allow'?
        , name.preferred_name AS givenName
        , name.surname AS familyName
        , NULL AS middlename
        , 'student' AS role
        , pupil.code AS identifier
        , name.email_address AS email
        , NULL AS sms
        , NULL AS phone
        -- , NULL AS agents
        , (
            SELECT cast(relationship.to_name_id AS varchar(36)) AS sourcedId
            FROM dbo.name na
                INNER JOIN dbo.relationship ON na.name_id = relationship.to_name_id
                INNER JOIN dbo.relationship_type ON relationship_type.id = relationship.relation_id
                INNER JOIN dbo.pupil AS p ON p.name_id = relationship.from_name_id
            WHERE pupil.name_id = relationship.from_name_id
                AND relationship.rank <= '2'
                AND relationship_type.to_relation != 'pupil'
                AND p.last_amend_date > @p1
                AND p.academic_year = @p2
                AND na.email_address IS NOT NULL
            FOR json path
        ) AS agents
        , (
            SELECT cast(school.school_id AS varchar(36)) AS sourcedId
            FROM dbo.school WHERE pupil.school = school.code FOR JSON PATH
        ) AS orgs
        , JSON_QUERY(CONCAT(
                '["'
                , form_year.age_range
                , '"]'
        )) AS grades
        , NULL AS password
    FROM dbo.pupil
        INNER JOIN dbo.name ON pupil.name_id = name.name_id
        INNER JOIN dbo.form ON pupil.form = form.code
        INNER JOIN dbo.form_year ON form.year_code = form_year.code
    WHERE pupil.last_amend_date > @p1
        AND pupil.academic_year = @p2
        AND form.academic_year = @p2
        AND pupil.record_type = 1
    ORDER BY sourcedId
    FOR JSON PATH, root('users')
    ) AS nvarchar(max)
) AS 'users'
-- name: select-users-teacher
UNION
SELECT cast((
    SELECT
        cast(staff.name_id AS varchar(36)) AS sourcedId
        , CASE WHEN staff.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(staff.last_amend_date AS datetimeoffset) AS dateLastModified
        , CASE WHEN staff.internal_email_address IS NULL THEN staff.code ELSE staff.internal_email_address END AS username
        , NULL AS userIds -- GUIDRef[0..*]
        , CASE WHEN staff.in_use = 'Y' THEN 1 ELSE 0 END AS enabledUser
        , name.preferred_name AS givenName
        , name.surname AS familyName
        , NULL AS middlename
        , 'teacher' AS role
        , staff.code AS identifier
        , staff.internal_email_address AS email
        , NULL AS sms
        , NULL AS phone
        , NULL AS agentSourcedIds -- GUIDRef[0..*]
        , cast(school.school_id AS varchar(36)) AS 'org.sourcedId' -- GUIDRef[1..*]
        , NULL AS grades
        , NULL AS password
    FROM dbo.staff
        INNER JOIN dbo.name ON name.name_id = staff.name_id
        INNER JOIN dbo.school ON school.code = staff.school
    WHERE staff.last_amend_date > @p1
        AND name.preferred_name IS NOT NULL -- handle service accounts
        AND (staff.category = 'TEA001' OR staff.category = 'SUPPLY' OR staff.category = 'EARLY')
    ORDER BY sourcedId
    FOR JSON PATH, root('users')
    ) AS nvarchar(max)
)
-- name: select-users-support-staff
UNION
SELECT cast((
    SELECT
        cast(staff.name_id AS varchar(36)) AS sourcedId
        , CASE WHEN staff.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(staff.last_amend_date AS datetimeoffset) AS dateLastModified
        , CASE WHEN staff.internal_email_address IS NULL THEN staff.code ELSE staff.internal_email_address END AS username
        , NULL AS userIds -- GUIDRef[0..*]
        , CASE WHEN staff.in_use = 'Y' THEN 1 ELSE 0 END AS enabledUser
        , name.preferred_name AS givenName
        , name.surname AS familyName
        , NULL AS middlename
        , 'aide' AS role
        , staff.code AS identifier
        , staff.internal_email_address AS email
        , NULL AS sms
        , NULL AS phone
        , NULL AS agentSourcedIds -- GUIDRef[0..*]
        , cast(school.school_id AS varchar(36)) AS 'org.sourcedId' -- GUIDRef[1..*]
        , NULL AS grades
        , NULL AS password
    FROM dbo.staff
        INNER JOIN dbo.name ON name.name_id = staff.name_id
        INNER JOIN dbo.school ON school.code = staff.school
    WHERE staff.last_amend_date > @p1
        AND name.preferred_name IS NOT NULL -- handle service accounts
        AND (staff.category = 'NON001' OR staff.category = 'COACH')
    ORDER BY sourcedId
    FOR JSON PATH, root('users')
    ) AS nvarchar(max)
)
-- name: select-users-parents
UNION
SELECT cast((
    SELECT
        cast(name.name_id AS varchar(36)) AS sourcedId
        , CASE WHEN name.contact_in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(name.amend_date AS datetimeoffset) AS dateLastModified
        , name.email_address AS username
        , NULL AS userIds
        , CASE WHEN name.contact_in_use = 'Y' THEN 1 ELSE 0 END AS enabledUser
        , name.preferred_name AS givenName
        , name.surname AS familyName
        , NULL AS middleName
        , CASE
            WHEN EXISTS (
                SELECT 1
                FROM dbo.name AS nn
                    INNER JOIN dbo.relationship ON name.name_id = relationship.to_name_id
                    INNER JOIN dbo.relationship_type ON relationship.relation_id = relationship_type.id
                WHERE relationship_type.to_relation = 'mother'
                    OR relationship_type.to_relation = 'father'
            ) THEN 'parent'
            WHEN EXISTS (
                SELECT 1
                FROM dbo.name AS nn
                    INNER JOIN dbo.relationship ON name.name_id = relationship.to_name_id
                    INNER JOIN dbo.relationship_type ON relationship.relation_id = relationship_type.ID
                WHERE relationship_type.to_relation = 'guardian'
            ) THEN 'guardian'
            ELSE 'relative'
        END AS role
        , name.name_code AS identifier
        , name.email_address AS email
        , NULL AS sms
        , NULL AS phone
        --, NULL AS agents
        , (
            SELECT cast(relationship.from_name_id AS varchar(36)) AS sourcedId
            --, pupil.ACADEMIC_YEAR 'academic year'
            FROM dbo.relationship
                INNER JOIN dbo.pupil ON relationship.from_name_id = pupil.name_id
                INNER JOIN dbo.form on form.code = pupil.form
            WHERE relationship.to_name_id = name.name_id
                AND pupil.academic_year = @p2
                AND pupil.record_type = 1
                AND form.academic_year = @p2
            FOR JSON PATH
        ) AS agents
        , (
            SELECT cast(school.school_id AS varchar(36)) AS sourcedId
            FROM dbo.relationship
                INNER JOIN dbo.pupil ON relationship.from_name_id = pupil.name_id
                INNER JOIN dbo.school ON pupil.school = school.code
            WHERE relationship.to_name_id = name.name_id
            GROUP BY school.school_id
            FOR JSON PATH
        ) AS orgs
        , NULL AS grades
        , NULL AS password
    FROM dbo.name
    -- validate user is a primary contact and not pupil
    -- no 'role' flag or column for validating parent/guardian status within pass
    WHERE EXISTS (
        SELECT 1
        FROM dbo.name na
            INNER JOIN dbo.relationship ON na.name_id = relationship.to_name_id
            INNER JOIN dbo.relationship_type ON relationship.relation_id = relationship_type.id
            INNER JOIN dbo.pupil ON pupil.name_id = relationship.from_name_id
        WHERE name.name_id = relationship.to_name_id
            AND relationship.rank <= 2
            AND relationship_type.to_relation != 'pupil'
            AND pupil.last_amend_date > @p1
            AND pupil.academic_year = @p2
    )
        AND name.email_address IS NOT NULL
    ORDER BY sourcedId
    FOR JSON PATH, root('users')
    ) AS nvarchar(max)
)
"#;

//TODO: add delta to enrollments
static QUERY_ENROLLMENTS: &str = r#"
-- name: select-enrollments-scheduled-pupil
SELECT cast((
    SELECT
        cast(pupil_set.pupil_set_id AS varchar(36)) AS sourcedId
        , CASE WHEN subject_set.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(getdate() as datetimeoffset) AS dateLastModified
        , cast(pupil.name_id AS varchar(36)) AS 'user.sourcedId'
        , cast(pupil_set.subject_set_id AS varchar(36)) AS 'class.sourcedId'
        , cast(school.school_id AS varchar(36)) AS 'school.sourcedId'
        , 'student' AS role
        , 0 AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM dbo.pupil_set
        INNER JOIN dbo.subject_set ON subject_set.subject_set_id = pupil_set.subject_set_id
        INNER JOIN dbo.school ON school.code = subject_set.school
        INNER JOIN dbo.pupil ON pupil.pupil_id = pupil_set.pupil_id
    WHERE subject_set.academic_year = @p2
    ORDER BY sourcedId
    FOR JSON PATH, root('enrollments')
    ) AS nvarchar(max)
) AS enrollments
-- name: select-enrollments-homeroom-pupil
UNION
SELECT cast((
    SELECT
        cast(concat(form.form_id, pupil.pupil_id) AS varchar(36)) AS sourcedId
        , CASE WHEN form.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(getdate() AS datetimeoffset) AS dateLastModified
        , cast(pupil.name_id AS varchar(36)) AS 'user.sourcedId'
        , cast(form.form_id AS varchar(36)) AS 'class.sourcedId'
        , cast(school.school_id AS varchar(36)) AS 'school.sourcedId'
        , 'student' AS role
        , 0 AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM dbo.PUPIL
        INNER JOIN dbo.form ON form.code = pupil.form
        INNER JOIN dbo.school ON school.code = pupil.school
    WHERE form.academic_year = @p2
        and pupil.academic_year = @p2
        and pupil.record_type = 1
    ORDER BY sourcedId
    FOR JSON PATH, root('enrollments')
    ) AS nvarchar(max)
)
-- name: select-enrollments-homeroom-teacher
UNION
SELECT cast((
    SELECT
       cast(concat(form.form_id, staff.name_id) AS varchar(36)) AS sourcedId
        , CASE WHEN form.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(getdate() AS datetimeoffset) AS dateLastModified
        , cast(staff.name_id AS varchar(36)) AS 'user.sourcedId'
        , cast(form.form_id AS varchar(36)) AS 'class.sourcedId'
        , cast(school.school_id AS varchar(36)) AS 'school.sourcedId'
        ,'teacher' AS role
        , 1 AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM dbo.form
        INNER JOIN dbo.staff ON form.tutor = staff.code
        INNER JOIN dbo.school ON school.code = staff.school
    WHERE form.academic_year = @p2
    ORDER BY sourcedId
    FOR JSON PATH, root('enrollments')
    ) AS nvarchar(max)
)
-- name: select-enrollments-scheduled-teacher-1
UNION
SELECT cast((
    SELECT
        cast(concat(subject_set.subject_set_id, staff.name_id) AS varchar(36)) AS sourcedId
        , CASE WHEN subject_set.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(getdate() AS datetimeoffset) AS dateLastModified
        , cast(staff.name_id AS varchar(36)) AS 'user.sourcedId'
        , cast(subject_set.subject_set_id AS varchar(36)) AS 'class.sourcedId'
        , cast(school.school_id AS varchar(36)) AS 'school.sourcedId'
        , 'teacher' AS role
        , 1 AS 'primary'
        , NULL AS begindate
        , NULL AS endDate
    FROM dbo.subject_set
        INNER JOIN dbo.staff ON subject_set.tutor = staff.code
        INNER JOIN dbo.school ON school.code = subject_set.school
    WHERE subject_set.academic_year = @p2
    ORDER BY sourcedId
    FOR JSON PATH, root('enrollments')
    ) AS nvarchar(max)
)
-- name: select-enrollments-scheduled-teacher-2
UNION
SELECT cast((
    SELECT
        cast(concat(subject_set.subject_set_id, staff.name_id) AS varchar(36)) AS sourcedId
        , CASE WHEN subject_set.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(getdate() AS datetimeoffset) AS dateLastModified
        , cast(staff.name_id AS varchar(36)) AS 'user.sourcedId'
        , cast(subject_set.subject_set_id AS varchar(36)) AS 'class.sourcedId'
        , cast(school.school_id AS varchar(36)) AS 'school.sourcedId'
        , 'teacher' AS role
        , 0 AS 'primary'
        , NULL AS begindate
        , NULL AS endDate
    FROM dbo.subject_set
        INNER JOIN dbo.staff ON subject_set.tutor_2 = staff.code
        INNER JOIN dbo.school ON school.code = subject_set.school
    WHERE subject_set.academic_year = @p2
    ORDER BY sourcedId
    FOR JSON PATH, root('enrollments')
    ) AS nvarchar(max)
)
-- name: select-enrollments-scheduled-teacher-3
UNION
SELECT cast((
    SELECT
        cast(concat(subject_set.subject_set_id, staff.name_id) AS varchar(36)) AS sourcedId
        , CASE WHEN subject_set.in_use = 'Y' THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(getdate() AS datetimeoffset) AS dateLastModified
        , cast(staff.name_id AS varchar(36)) AS 'user.sourcedId'
        , cast(subject_set.subject_set_id AS varchar(36)) AS 'class.sourcedId'
        , cast(school.school_id AS varchar(36)) AS 'school.sourcedId'
        , 'teacher' AS role
        , 0 AS 'primary'
        , NULL AS begindate
        , NULL AS endDate
    FROM dbo.subject_set
        INNER JOIN dbo.staff ON subject_set.tutor_3 = staff.code
        INNER JOIN dbo.school ON school.code = subject_set.school
    WHERE subject_set.academic_year = @p2
    ORDER BY sourcedId
    FOR JSON PATH, root('enrollments')
    ) AS nvarchar(max)
)
"#;
