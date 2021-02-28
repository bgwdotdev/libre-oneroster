//use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/*
struct GUIDRef {
    sourced_id: String,
    _type: String,
}

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
    status: String,
    year: Option<String>,
}

/*
enum Status {
    Active(String),
    ToBeDeleted(String),
}

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
