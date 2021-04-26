//use chrono::{DateTime, Utc};
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

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GUIDRef {
    pub sourced_id: String,
    pub _type: String,
}

/*
pub struct AcademicSessions {
    sourced_id: String,
    status: String,
    date_last_modified: DateTime<Utc>,
    title: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    _type: SessionType, //review name
    parent: GUIDRef,
    children: Vec<GUIDRef>,
    school_year: String,
}
*/

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademicSession {
    pub sourced_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Org {
    pub sourced_id: String,
    pub status: Status,
    pub name: String,
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "vec_is_none")]
    pub children: Option<Vec<Option<String>>>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
// required due to writing to db as
// UpperCamelCase, causing serialization errors
#[allow(non_camel_case_types)]
pub enum Status {
    active,
    tobedeleted,
}

/*
enum ClassType {
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
