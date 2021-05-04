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
/*
enum ClassType {
    #[allow(non_camel_case_types)]
    HomeRoom,
    Scheduled,
}

enum SessionType {
    GradingPeriod,
    Semester,
    SchoolYear,
    Term,
}
*/
