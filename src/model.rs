use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

// Evaluates if the first item is None in a vector
// Used to skip serializing an empty sub item in a json struct
fn vec_is_none<T>(v: &Option<Vec<Option<T>>>) -> bool {
    if let Some(i) = v {
        if i[0].is_none() {
            return true;
        }
    }
    false
}

// while href and ref_type are not optional
// in the spec output, they are for the purposes
// of ingest. Their required state is enforced by
// the accompanying sql query
#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct GUIDRef {
    pub href: Option<String>,
    pub sourced_id: String,
    #[serde(rename = "type")]
    pub ref_type: Option<GUIDType>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicSession {
    pub sourced_id: String,
    pub status: StatusType,
    pub date_last_modified: DateTime<Utc>,
    pub title: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    #[serde(rename = "type")]
    pub academic_session_type: SessionType,
    pub parent: Option<GUIDRef>,
    pub children: Option<Vec<GUIDRef>>,
    pub school_year: i32,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub sourced_id: String,
    pub status: StatusType,
    pub date_last_modified: DateTime<Utc>,
    pub title: String,
    pub period_code: String,
    pub description: Option<String>,
    pub orgs: Vec<GUIDRef>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub sourced_id: String,
    pub status: StatusType,
    pub date_last_modified: DateTime<Utc>,
    pub title: String,
    pub subject_code: String,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Class {
    pub sourced_id: String,
    pub status: StatusType,
    pub date_last_modified: DateTime<Utc>,
    pub title: String,
    pub class_code: Option<String>,
    pub class_type: ClassType,
    pub location: Option<String>,
    pub grades: Option<Vec<String>>,
    pub subjects: Option<Vec<String>>,
    pub course: GUIDRef,
    pub school: GUIDRef,
    pub terms: Vec<GUIDRef>,
    pub subject_codes: Option<Vec<String>>,
    pub periods: Option<Vec<String>>,
    pub resources: Option<Vec<GUIDRef>>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub sourced_id: String,
    pub status: StatusType,
    pub date_last_modified: DateTime<Utc>,
    pub title: String,
    pub school_year: Option<GUIDRef>,
    pub course_code: Option<String>,
    pub grades: Option<Vec<String>>,
    pub subjects: Option<Vec<String>>,
    pub org: GUIDRef,
    pub subject_codes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Org {
    pub sourced_id: String,
    pub status: StatusType,
    pub date_last_modified: DateTime<Utc>,
    pub name: String,
    #[serde(rename = "type")]
    pub org_type: OrgType,
    pub identifier: Option<String>,
    pub parent: Option<GUIDRef>,
    pub children: Option<Vec<GUIDRef>>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub sourced_id: String,
    pub status: StatusType,
    pub date_last_modified: DateTime<Utc>,
    pub username: String,
    pub user_ids: Option<Vec<UserId>>,
    pub enabled_user: i8,
    pub given_name: String,
    pub family_name: String,
    pub middle_name: Option<String>,
    pub role: RoleType,
    pub identifier: Option<String>,
    pub email: Option<String>,
    pub sms: Option<String>,
    pub phone: Option<String>,
    pub agents: Option<Vec<GUIDRef>>,
    pub orgs: Option<Vec<GUIDRef>>,
    pub grades: Option<Vec<String>>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct UserId {
    #[serde(rename = "type")]
    pub id_type: String,
    pub identifier: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
// required due to writing to db as
// UpperCamelCase, causing serialization errors
#[allow(non_camel_case_types)]
pub enum StatusType {
    active,
    tobedeleted,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[allow(non_camel_case_types)]
pub enum OrgType {
    department,
    school,
    district,
    local,
    state,
    national,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[allow(non_camel_case_types)]
pub enum SessionType {
    gradingPeriod,
    semester,
    schoolYear,
    term,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[allow(non_camel_case_types)]
pub enum RoleType {
    administrator,
    aide,
    guardian,
    parent,
    proctor,
    relative,
    student,
    teacher,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[allow(non_camel_case_types)]
pub enum GUIDType {
    academicSession,
    category,
    class,
    course,
    demographics,
    enrollment,
    gradingPeriod,
    lineItem,
    org,
    resource,
    result,
    student,
    teacher,
    term,
    user,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[allow(non_camel_case_types)]
pub enum ClassType {
    homeroom,
    scheduled,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Classes {
    pub classes: Vec<Class>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct AcademicSessions {
    pub academic_sessions: Vec<AcademicSession>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Periods {
    pub periods: Vec<Period>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Orgs {
    pub orgs: Vec<Org>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Users {
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Subjects {
    pub subjects: Vec<Subject>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
pub struct Courses {
    pub courses: Vec<Course>,
}
