static QUERY_ORGS: &str = r#"
SELECT cast((
    SELECT
        cast(TblSchoolManagementSchoolSetupId AS varchar(36)) AS sourcedId
        , 'active' AS status
        , cast(dteSubmitDateTime AS datetimeoffset) AS dateLastModified
        , txtSchoolName AS name
        , 'school' AS 'type'
        , NULL AS identifier -- Programmatic name? TGA001?
        , NULL AS parent
        , NULL AS children
    FROM dbo.TblSchoolManagementSchoolSetup
    --WHERE dteSubmitDateTime > @p1
    ORDER BY sourcedId
    FOR JSON PATH, root('orgs')
    ) AS nvarchar(max)
)
"#;

// No content in TblSchoolManagementYear for 'schoolYear' type?
static QUERY_ACADEMIC_SESSIONS: &str = r#"
SELECT cast((
    SELECT
        cast(TblSchoolManagementTermDatesID AS varchar(36)) AS sourcedId
        , 'active' AS status
        , cast(txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , concat(intSchoolYear,TermNames.txtName) AS title
        , cast(txtStartDate AS date) AS startDate
        , cast(txtFinishDate AS date) AS endDate
        , 'term' AS 'type'
        , cast(intSchoolYear AS varchar(4)) AS schoolYear
    FROM TblSchoolManagementTermDates AS TermDates
        INNER JOIN TblSchoolManagementTermNames AS TermNames ON TermDates.intTerm = TermNames.TblSchoolManagementTermNamesID
    WHERE dateLastModified > @p1
        AND schoolYear = @p2
    ORDER BY sourcedId
    FOR JSON PATH, root('academicSessions')
    ) AS nvarchar(max)
)
"#;

// NO SUBJECTS

static QUERY_COURSES: &str = r#"
SELECT cast((
   SELECT
        cast(TblTeachingManagerSubjectsID AS varchar(36)) AS sourcedId
        , CASE WHEN intActive = 1 THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , txtSubjectName AS title
        , NULL AS schoolYear
        , txtSubjectCode AS courseCode
        , NULL AS grades
        , NULL AS subjects
        , cast((SELECT TOP(1) TblSchoolManagementSchoolSetupId FROM TblSchoolManagementSchoolSetup) AS varchar(36)) AS 'org.sourcedId' -- TODO: better alternative?
        , NULL AS subjectCodes
    FROM TblTeachingManagerSubjects
    WHERE txtSubmitDateTime > @p1
    ORDER BY sourcedId
    FOR JSON PATH, root('courses') 
    ) AS nvarchar(max)
)
"#;

static QUERY_CLASSES: &str = r#"
SELECT cast((
    SELECT
        cast(TblTeachingManagerSetsID AS varchar(36)) AS sourcedId
        , CASE WHEN blnActive = 1 THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , txtName AS title
        , txtSetCode AS classCode
        , 'scheduled' AS classType -- TODO: review if sets include tutor groups?
        , NULL AS location -- TODO: intClassroom join?
        , json_query(concat('["',intYear,'"]')) AS grades -- TODO: convert to CEDS
        , NULL AS subjects
        , cast(intSubject AS varchar(36)) AS 'course.sourcedId'
        , cast((SELECT TOP(1) TblSchoolManagementSchoolSetupId FROM TblSchoolManagementSchoolSetup) AS varchar(36)) AS 'org.sourcedId' -- TODO: better alternative?
        , '2020' AS 'terms.sourcedId' -- TODO: link to terms/academicSessions GUIDREF[1..*]
        , NULL AS subjectCodes
        , NULL AS periods
    FROM TblTeachingManagerSets
    WHERE txtSubmitDateTime > @p1
    ORDER BY sourcedId
    FOR JSON PATH, root('classes')
    ) AS nvarchar(max)
)
UNION
SELECT cast((
    SELECT
        cast(TblTeachingManagerSetsID AS varchar(36)) AS sourcedId
        , CASE WHEN blnActive = 1 THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , txtName AS title
        , txtSetCode AS classCode
        , 'scheduled' AS classType -- TODO: review if sets include tutor groups?
        , NULL AS location -- TODO: intClassroom join?
        , json_query(concat('["',intYear,'"]')) AS grades -- TODO: convert to CEDS
        , NULL AS subjects
        , cast(intSubject AS varchar(36)) AS 'course.sourcedId'
        , cast((SELECT TOP(1) TblSchoolManagementSchoolSetupId FROM TblSchoolManagementSchoolSetup) AS varchar(36)) AS 'org.sourcedId' -- TODO: better alternative?
        , '2020' AS 'terms.sourcedId' -- TODO: link to terms/academicSessions GUIDREF[1..*]
        , NULL AS subjectCodes
        , NULL AS periods
    FROM TblTeachingManagerSets
    WHERE dateLastModified > @p1
    ORDER BY sourcedId
    FOR JSON PATH, root('classes')
    ) AS nvarchar(max)
)
"#;

static QUERY_USERS: &str = r#"
-- pupils

DROP TABLE IF EXISTS #OnerosterParentsPupils;

-- Temp table to split parent contacts up from one row to two
-- TODO: merge parent various contacts?
SELECT addresses.intPersonId AS parentId, pupils.txtSchoolID AS pupilId
INTO #OnerosterParentsPupils
FROM TblPupilManagementAddresses addresses
	INNER JOIN TblPupilManagementAddressLink addressLink ON addressLink.intAddressID = addresses.TblPupilManagementAddressesID
	INNER JOIN TblPupilManagementPupils pupils ON pupils.txtSchoolID = addressLink.txtSchoolID
WHERE addresses.intPersonId IS NOT NULL
	AND addresses.txtAddressType = 'Home'
	AND (addresses.intMailMergeAll = 1 OR addresses.intCorrespondenceMailMerge = 1)
UNION
SELECT addresses.intSecondaryPersonId, pupils.txtSchoolID 
FROM TblPupilManagementAddresses addresses
	INNER JOIN TblPupilManagementAddressLink addressLink  ON addressLink.intAddressID = addresses.TblPupilManagementAddressesID
	INNER JOIN TblPupilManagementPupils pupils ON pupils.txtSchoolID = addressLink.txtSchoolID
WHERE addresses.intSecondaryPersonId IS NOT NULL
	AND addresses.txtAddressType = 'Home'
	AND (addresses.intMailMergeAll = 1 OR addresses.intCorrespondenceMailMerge = 1)
;

SELECT 
	cast(pupils.intPersonId AS varchar(36)) AS sourcedId -- TlbPupilManagementPupilsId does not change on end of year. TODO: use schoolId (for joins?)?
	, CASE WHEN intSystemStatus = -1 THEN 'tobedeleted' ELSE 'active' END AS status
	, cast(pupils.txtSubmitDateTime AS datetimeoffset) AS dateLastModified
	, pupils.txtEmailAddress AS username -- TODO: handle NULLS
	, NULL AS userIds
	, pupils.intSystemStatus AS enabledUser
	, pupils.txtPreName AS givenName
	, pupils.txtSurname AS familyName
	, NULL AS middlename -- , pupils.txtMiddleNames AS middlename
	, 'student' AS role
	, pupils.txtUserCode AS identifier
	, pupils.txtEmailAddress AS email
	, NULL AS sms
	, NULL AS phone
	, ( 
		SELECT cast(parents.parentId AS varchar(36)) AS sourcedId 
		FROM #OnerosterParentsPupils parents
		WHERE pupils.txtSchoolId = parents.pupilId
		FOR JSON PATH
	) AS agents
	, (
		SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36)) AS sourcedId
		FROM TblSchoolManagementSchoolSetup
		FOR JSON PATH
	) AS orgs
	, CASE WHEN intNCYear IS NOT NULL THEN json_query(concat('["',intNCYear,'"]')) ELSE NULL END AS grades -- TODO: convert to CEDS
	, NULL AS password
FROM TblPupilManagementPupils pupils
WHERE intSystemStatus = 1 -- TODO: Remove & WHERE dateLastModified IS NOT NULL?
	OR intSystemStatus = 0 -- 1 current pupil / -1 leaver / 0 To start
ORDER BY sourcedId
FOR JSON PATH

"#;
