pub static QUERY_ORGS: &str = r#"
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
    WHERE dteSubmitDateTime > @p1
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER -- root('orgs')
    ) AS nvarchar(max)
) AS orgs
"#;

// No content in TblSchoolManagementYear for 'schoolYear' type?
pub static QUERY_ACADEMIC_SESSIONS: &str = r#"
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
    WHERE txtSubmitDateTime > @p1
        --AND schoolYear = @p2
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER -- root('academicSessions')
    ) AS nvarchar(max)
) AS academicSessions
"#;

// NO SUBJECTS

pub static QUERY_COURSES: &str = r#"
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
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'org.sourcedId'
        , NULL AS subjectCodes
    FROM TblTeachingManagerSubjects
    WHERE txtSubmitDateTime > @p1
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER -- root('courses')
    ) AS nvarchar(max)
) AS courses
"#;

pub static QUERY_CLASSES: &str = r#"
-- Scheduled
SELECT cast((
    SELECT
        cast(TblTeachingManagerSetsID AS varchar(36)) AS sourcedId
        , CASE WHEN blnActive = 1 THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(sets.txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , txtName AS title
        , txtSetCode AS classCode
        , 'scheduled' AS classType
        , NULL AS location
        , json_query(concat('["',years.txtWebsite,'"]')) AS grades -- CHANGE ME: CEDS value
        , NULL AS subjects
        , cast(intSubject AS varchar(36)) AS 'course.sourcedId'
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'school.sourcedId'
        , ( 
            SELECT '1' AS sourcedId -- TODO: link to terms/academicSessions GUIDREF[1..*]
            FOR JSON PATH
        ) AS terms
        , NULL AS subjectCodes
        , NULL AS periods
    FROM TblTeachingManagerSets sets
        INNER JOIN TblSchoolManagementYears years ON years.intNCYear = sets.intYear
    WHERE sets.txtSubmitDateTime > @p1
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('classes')
    ) AS nvarchar(max)
) AS classes
-- Homerooms
UNION
SELECT cast((
    SELECT
        cast(txtForm AS varchar(36)) AS sourcedId
        , 'active' AS status
        , cast(forms.txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , txtForm AS title
        , txtForm AS classCode
        , 'homeroom' AS classType
        , NULL AS location
        , json_query(concat('["',years.txtWebsite,'"]')) AS grades -- CHANGE ME: CEDS value
        , NULL AS subjects
        , (
            SELECT cast(TblTeachingManagerSubjectsId AS varchar(36))
            FROM TblTeachingManagerSubjects
            WHERE txtSubjectName = 'Tutor Group'
        ) AS 'course.sourcedId'
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'school.sourcedId'
        , (
            SELECT '1' AS sourcedId -- TODO: link to terms/academicSessions GUIDREF[1..*]
            FOR JSON PATH
        ) AS terms
        , NULL AS subjectCodes
        , NULL AS periods
    FROM TblSchoolManagementForms forms
        INNER JOIN TblSchoolManagementYears years ON years.intNCYear = forms.intNCYear
    WHERE forms.txtSubmitDateTime > @p1
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('classes')
    ) AS nvarchar(max)
)
"#;

pub static QUERY_USERS: &str = r#"

DROP TABLE IF EXISTS #OnerosterPupilParents;

-- Temp table to split out dual parent contacts into seperate rows
-- TODO: merge parent various contacts?
SELECT addresses.intPersonId AS parentId, pupils.txtSchoolID AS pupilId
INTO #OnerosterPupilParents
FROM TblPupilManagementAddresses addresses
    INNER JOIN TblPupilManagementAddressLink addressLink ON addressLink.intAddressID = addresses.TblPupilManagementAddressesID
    INNER JOIN TblPupilManagementPupils pupils ON pupils.txtSchoolID = addressLink.txtSchoolID
WHERE addresses.intPersonId IS NOT NULL
    AND addresses.txtContactsForename <> ''
    AND addresses.txtAddressType = 'Home'
    AND (addresses.intMailMergeAll = 1 OR addresses.intCorrespondenceMailMerge = 1)
    AND pupils.txtSubmitDateTime > @p1
    AND pupils.txtEmailAddress IS NOT NULL
    AND ( pupils.intSystemStatus = 1 OR pupils.intSystemStatus = 0 )
    AND addresses.intPrivate = 0
    AND datalength(addresses.txtEmail1) > 0
    AND addresses.txtSubmitDateTime > @p1
UNION
SELECT addresses.intSecondaryPersonId, pupils.txtSchoolID
FROM TblPupilManagementAddresses addresses
    INNER JOIN TblPupilManagementAddressLink addressLink  ON addressLink.intAddressID = addresses.TblPupilManagementAddressesID
    INNER JOIN TblPupilManagementPupils pupils ON pupils.txtSchoolID = addressLink.txtSchoolID
WHERE addresses.intSecondaryPersonId IS NOT NULL
    AND addresses.txtSecondaryForename <> ''
    AND addresses.txtAddressType = 'Home'
    AND (addresses.intMailMergeAll = 1 OR addresses.intCorrespondenceMailMerge = 1)
    AND pupils.txtSubmitDateTime > @p1
    AND pupils.txtEmailAddress IS NOT NULL
    AND ( pupils.intSystemStatus = 1 OR pupils.intSystemStatus = 0 )
    AND addresses.intPrivate = 0
    AND datalength(addresses.txtEmail2) > 0
    AND addresses.txtSubmitDateTime > @p1
;

-- pupils
SELECT cast((
    SELECT
        cast(pupils.txtSchoolId AS varchar(36)) AS sourcedId
        , CASE WHEN intSystemStatus = -1 THEN 'tobedeleted' ELSE 'active' END AS status
        , cast(pupils.txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , CASE WHEN pupils.txtEmailAddress IS NULL THEN txtSchoolId ELSE pupils.txtEmailAddress END AS username
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
            FROM #OnerosterPupilParents parents
            WHERE pupils.txtSchoolId = parents.pupilId
            FOR JSON PATH
        ) AS agents
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36)) AS sourcedId
            FROM TblSchoolManagementSchoolSetup
            FOR JSON PATH
        ) AS orgs
        , CASE WHEN pupils.intNCYear IS NOT NULL THEN json_query(concat('["',years.txtWebsite,'"]')) ELSE NULL END AS grades -- CHANGE ME: CEDS value
        , NULL AS password
    FROM TblPupilManagementPupils pupils
		INNER JOIN TblSchoolManagementYears years ON years.intNCYear = pupils.intNCYear
    WHERE pupils.txtSubmitDateTime > @p1
        --AND pupils.txtEmailAddress IS NOT NULL
        -- TODO: Remove & WHERE dateLastModified IS NOT NULL?
        -- 1 current pupil / -1 leaver / 0 To start
        AND ( intSystemStatus = 1 OR intSystemStatus = 0 )
        AND pupils.txtPreName <> ''
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('users')
    ) AS nvarchar(max)
) AS users

UNION
-- staff
SELECT cast((
    SELECT
        cast(TblStaffId AS varchar(36)) AS sourcedId -- TODO: consider GUIDUniquePersonId ?
        , CASE WHEN SystemStatus = -1 THEN 'tobedeleted' ELSE 'active' END AS status
        , cast(SubmitDate AS datetimeoffset) AS dateLastModified
        , CASE WHEN schoolEmailAddress <> '' THEN schoolEmailAddress  ELSE  cast(TblStaffId AS varchar(36)) END AS username -- TODO: username null?
        , NULL AS userIds
        , CASE WHEN SystemStatus = 1 THEN 1 ELSE 0 END AS enabledUser
        , PreName AS givenName
        , Surname AS familyName
        , NULL AS middlename
        , CASE WHEN TeachingStaff = 1 THEN 'teacher' ELSE 'aide' END AS role
        , User_Code AS identifier
        , schoolEmailAddress AS email
        , NULL AS sms
        , NULL AS phone
        , NULL AS agents
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36)) AS sourcedId
            FROM TblSchoolManagementSchoolSetup
            FOR JSON PATH
        ) AS orgs
        , NULL AS grades
        , NULL AS password
    FROM TblStaff
    WHERE SubmitDate IS NOT NULL
        AND SubmitDate > @p1
        AND PreName <> '' -- deals with service accounts
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('users')
    ) AS nvarchar(max)
)

UNION
-- parent 1
SELECT cast((
    SELECT 
        cast(intPersonId AS varchar(36)) AS sourcedId
        , 'active' AS status
        , cast(txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , txtEmail1 AS username
        , NULL AS userIds
        , 1 AS enabledUser
        , txtContactsForename AS givenName
        , txtContactsSurname AS familyName
        , NULL AS middlename
        , CASE
            WHEN txtRelationType = 'Parents'
                OR txtRelationType = 'Father'
                OR txtRelationType = 'Mother'
                OR txtRelationType = 'Mother and Stepfather'
                OR txtRelationType = 'Father and Stepmother'
                OR txtRelationType = 'Mothers''s Partner'
                OR txtRelationType = 'parent'
                OR txtRelationType = 'Father and Father'
            THEN 'parent'
            WHEN txtRelationType = 'Grandmother'
                OR txtRelationType = 'Grandparents'
                OR txtRelationType = 'Uncle'
                OR txtRelationType = 'Aunt'
            THEN 'relative'
            WHEN txtRelationType = 'Guardian'
                OR txtRelationType = 'Guardians'
            THEN 'guardian'
            ELSE 'guardian' -- TODO: handle all cases
        END AS role
        , guidUniquePersonID AS identifier
        , txtEmail1 AS email
        , NULL AS sms
        , NULL AS phone
        , (
            SELECT cast(pupils.PupilId AS varchar(36)) AS sourcedId
            FROM #OnerosterPupilParents pupils
            WHERE intPersonId = pupils.ParentID
            FOR JSON PATH
        ) AS agents
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36)) AS sourcedId
            FROM TblSchoolManagementSchoolSetup
            FOR JSON PATH
        ) AS orgs
        , NULL AS grades
        , NULL AS password
    FROM TblPupilManagementAddresses
    WHERE intPersonId IS NOT NULL
        AND txtAddressType = 'Home'
        AND (intMailMergeAll = 1 OR intCorrespondenceMailMerge = 1)
		AND intPrivate = 0
		AND datalength(txtEmail1) > 0
        AND txtSubmitDateTime > @p1
	ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('users')
    ) AS nvarchar(max)
)
UNION
-- parent 2
SELECT cast((
    SELECT 
        cast(intSecondaryPersonId as varchar(36)) AS sourcedId
        , 'active' AS status
        , cast(txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , txtEmail2 AS username
        , NULL AS userIds
        , 1 AS enabledUser
        , txtSecondaryForename AS givenName
        , txtSecondarySurname AS familyName
        , NULL AS middlename
        , CASE
            WHEN txtRelationType = 'Parents'
                OR txtRelationType = 'Father'
                OR txtRelationType = 'Mother'
                OR txtRelationType = 'Mother and Stepfather'
                OR txtRelationType = 'Father and Stepmother'
                OR txtRelationType = 'Mothers''s Partner'
                OR txtRelationType = 'parent'
                OR txtRelationType = 'Father and Father'
            THEN 'parent'
            WHEN txtRelationType = 'Grandmother'
                OR txtRelationType = 'Grandparents'
                OR txtRelationType = 'Uncle'
                OR txtRelationType = 'Aunt'
            THEN 'relative'
            WHEN txtRelationType = 'Guardian'
                OR txtRelationType = 'Guardians'
            THEN 'guardian'
            ELSE 'guardian'
        END AS role
        , guidSecondaryUniquePersonID AS identifier
        , txtEmail2 AS email
        , NULL AS sms
        , NULL AS phone
        , (
            SELECT cast(pupils.PupilId AS varchar(36)) AS sourcedId
            FROM #OnerosterPupilParents pupils
            WHERE intSecondaryPersonId = pupils.ParentID
            FOR JSON PATH
        ) AS agents
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36)) AS sourcedId
            FROM TblSchoolManagementSchoolSetup
            FOR JSON PATH
        ) AS orgs
        , NULL AS grades
        , NULL AS password
    FROM TblPupilManagementAddresses
    WHERE intSecondaryPersonId IS NOT NULL
        AND txtAddressType = 'Home'
        AND (intMailMergeAll = 1 OR intCorrespondenceMailMerge = 1)
		AND intPrivate = 0
		AND datalength(txtEmail2) > 0
        AND txtSubmitDateTime > @p1
	ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('users')
    ) AS nvarchar(max)
)
"#;

pub static QUERY_ENROLLMENTS: &str = r#"
-- scheduled pupil
SELECT cast((
    SELECT
        --cast(TblTeachingManagerSetListsId AS varchar(36)) AS sourcedId
        cast(hashbytes('md5', concat(intSetId, setlists.txtSchoolId)) AS uniqueidentifier) AS sourcedId
        , CASE WHEN pupils.intSystemStatus = -1 THEN 'tobedeleted' ELSE 'active' END AS status
        , cast(setlists.txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , cast(setlists.txtSchoolId AS varchar(36)) AS 'user.sourcedId'
        , cast(intSetId AS varchar(36)) AS 'class.sourcedId'
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'school.sourcedId'
        , 'student' AS role
        , NULL AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM TblTeachingManagerSetLists setlists
        INNER JOIN TblPupilManagementPupils pupils ON pupils.txtSchoolId = setlists.txtSchoolId
    WHERE setlists.txtSubmitDateTime > @p1
        --AND pupils.txtSubmitDateTime > @p1 -- TODO: decide if want pupil status monitored
        AND pupils.txtEmailAddress IS NOT NULL
        AND pupils.txtPreName <> ''
        AND ( intSystemStatus = 1 OR intSystemStatus = 0 )
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('enrollments')
    ) AS nvarchar(max)
) AS enrollments
UNION
-- homeroom pupil
SELECT cast((
   SELECT
        cast(hashbytes('md5', concat(txtForm, txtSchoolId)) AS uniqueidentifier) AS sourcedId
		, CASE WHEN intSystemStatus = -1 THEN 'tobedeleted' ELSE 'active' END AS status -- TODO: verify behaviour with 0 users?
		, cast(txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , txtForm AS 'class.sourcedId' -- TODO: use intTagId?
        , txtSchoolId AS 'user.sourcedId'
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'school.sourcedId'
        , 'student' AS role
        , NULL AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM TblPupilManagementPupils
    WHERE txtForm IS NOT NULL
		AND txtSubmitDateTime > @p1
        AND txtEmailAddress IS NOT NULL
        AND txtPreName <> ''
        AND ( intSystemStatus = 1 OR intSystemStatus = 0 )
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('enrollments')
    ) AS nvarchar(max)
)
UNION
--scheduled teachers 1
SELECT cast((
    SELECT
        cast(hashbytes('md5', concat(TblTeachingManagerSetsId, staff.TblStaffId)) AS uniqueidentifier) AS sourcedId
        , CASE WHEN blnActive = 1 THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(sets.txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , cast(staff.TblStaffId AS varchar(36)) AS 'user.sourcedId'
        , cast(TblTeachingManagerSetsId AS varchar(36)) AS 'class.sourcedId'
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'school.sourcedId'
        , 'teacher' AS role
        , 1 AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM TblTeachingManagerSets sets
        INNER JOIN TblStaff staff ON staff.user_code = sets.txtTeacher
    WHERE txtSubmitDateTime > @p1
        AND staff.SubmitDate IS NOT NULL
        AND staff.SubmitDate > @p1
        AND schoolEmailAddress <> ''
        AND schoolEmailAddress IS NOT NULL
        AND PreName IS NOT NULL -- deals with service accounts
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('enrollments')
    ) AS nvarchar(max)
)
UNION
--scheduled teacher 2..
SELECT cast((
    SELECT
        cast(hashbytes('md5', concat(TblTeachingManagerSetsId, staff.TblStaffId)) AS uniqueidentifier) AS sourcedId
        , CASE WHEN blnActive = 1 THEN 'active' ELSE 'tobedeleted' END AS status
        , cast(txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , cast(staff.TblStaffId AS varchar(36)) AS 'user.sourcedId'
        , cast(TblTeachingManagerSetsId AS varchar(36)) AS 'class.sourcedId'
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'school.sourcedId'
        , 'teacher' AS role
        , 0 AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM TblTeachingManagerSetAssociatedTeachers
        INNER JOIN TblTeachingManagerSets ON TblTeachingManagerSets.TblTeachingManagerSetsId = TblTeachingManagerSetAssociatedTeachers.intSetId
        INNER JOIN TblStaff staff ON staff.user_code = TblTeachingManagerSetAssociatedTeachers.txtTeacher
    WHERE txtSubmitDateTime > @p1
        AND staff.SubmitDate IS NOT NULL
        AND schoolEmailAddress <> ''
        AND schoolEmailAddress IS NOT NULL
        AND PreName IS NOT NULL -- deals with service accounts
    ORDER BY sourcedId
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('enrollments')
    ) AS nvarchar(max)
)
UNION
-- enrollments homeroom teacher 1
SELECT cast((
    SELECT
        cast(hashbytes('md5', concat(txtForm, staff.TblStaffId)) AS uniqueidentifier) AS sourcedId
        , 'active' AS status
        , cast(forms.txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , cast(txtForm AS varchar(36)) AS 'class.sourcedId'
        , cast(staff.TblStaffId AS varchar(36)) AS 'user.sourcedId'
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'school.sourcedId'
        , 'teacher' AS role
        , 1 AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM TblSchoolManagementForms forms
        INNER JOIN TblStaff staff ON staff.user_code = forms.txtFormTutor
    WHERE forms.txtSubmitDateTime > @p1
        AND staff.SubmitDate IS NOT NULL
        AND schoolEmailAddress <> ''
        AND schoolEmailAddress IS NOT NULL
        AND PreName IS NOT NULL -- deals with service accounts
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('enrollments')
    ) AS nvarchar(max)
)
UNION
-- enrollments homeroom teacher 2
SELECT cast((
    SELECT
        cast(hashbytes('md5', concat(txtForm, TblStaffID)) AS uniqueidentifier) AS sourcedId
        , 'active' AS status
        , cast(txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , cast(txtForm AS varchar(36)) AS 'class.sourcedId'
        , cast(staff.TblStaffId AS varchar(36)) AS 'user.sourcedId'
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'school.sourcedId'
        , 'teacher' AS role
        , 0 AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM TblSchoolManagementForms forms
        INNER JOIN TblStaff staff ON staff.user_code = forms.txtAsstFormTutor
    WHERE forms.txtAsstFormTutor <> ''
        AND forms.txtSubmitDateTime > @p1
        AND staff.SubmitDate IS NOT NULL
        AND schoolEmailAddress <> ''
        AND schoolEmailAddress IS NOT NULL
        AND PreName IS NOT NULL -- deals with service accounts
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('enrollments')
    ) AS nvarchar(max)
)
UNION
-- enrollments homeroom teacher 3
SELECT cast((
    SELECT
        cast(hashbytes('md5', concat(txtForm, staff.TblStaffId)) AS uniqueidentifier) AS sourcedId
        , 'active' AS status
        , cast(forms.txtSubmitDateTime AS datetimeoffset) AS dateLastModified
        , cast(txtForm AS varchar(36)) AS 'class.sourcedId'
        , cast(staff.TblStaffId AS varchar(36)) AS 'user.sourcedId'
        , (
            SELECT cast(TblSchoolManagementSchoolSetupId AS varchar(36))
            FROM TblSchoolManagementSchoolSetup
        ) AS 'school.sourcedId'
        , 'teacher' AS role
        , 0 AS 'primary'
        , NULL AS beginDate
        , NULL AS endDate
    FROM TblSchoolManagementForms forms
        INNER JOIN TblStaff staff ON staff.user_code = forms.txtAsstFormTutor2
    WHERE forms.txtAsstFormTutor2 <> ''
        AND forms.txtSubmitDateTime > @p1
        AND staff.SubmitDate IS NOT NULL
        AND schoolEmailAddress <> ''
        AND schoolEmailAddress IS NOT NULL
        AND PreName IS NOT NULL -- deals with service accounts
    FOR JSON PATH, WITHOUT_ARRAY_WRAPPER --, root('enrollments')
    ) AS nvarchar(max)
)
"#;
