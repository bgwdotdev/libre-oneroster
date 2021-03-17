use super::{Result, State};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[serde(default)]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Parameters {
    pub(crate) limit: u32,  // 10
    pub(crate) offset: u32, // 20
    pub(crate) sort: Option<String>,
    pub(crate) filter: Option<String>, // name=bob AND age>20
    pub(crate) fields: Option<String>, // name,age
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
            sort: None,
            filter: None,
            fields: None,
        }
    }
}

pub(crate) async fn apply_parameters(json: &String, params: &Parameters) -> String {
    let q = parameter_builder(&params).await;
    let output = jq_rs::run(&q, &json).unwrap();
    output
}

async fn parameter_builder(params: &Parameters) -> String {
    let mut builder = format!("[ .[] ");

    if let Some(fields) = parse_fields(params).await {
        let f = format!("| {{ {} }} ", fields);
        builder.push_str(&f);
    }

    if let Some(filter) = parse_filter(params).await {
        let f = format!("| select({}) ", filter);
        builder.push_str(&f);
    }

    builder.push_str("] ");

    if let Some(sort) = parse_sort(params).await {
        let f = format!("| sort_by(.{}) ", sort);
        builder.push_str(&f);
    }

    log::debug!("parameter jq builder: {}", builder);
    builder
}

async fn parse_sort(params: &Parameters) -> Option<String> {
    if let Some(q_sort) = &params.sort {
        return Some(q_sort.to_owned());
    }
    None
}

async fn parse_filter(params: &Parameters) -> Option<String> {
    if let Some(q_filter) = &params.filter {
        let rlogic = Regex::new(r" (AND|OR) ").unwrap();
        let mut logicals: Vec<String> = Vec::new();
        for cap in rlogic.captures_iter(q_filter) {
            if &cap[1] == "AND" {
                logicals.push("and".to_string());
            } else {
                logicals.push("or".to_string());
            }
        }

        let raw_filters: Vec<&str> = rlogic.split(q_filter).collect();
        let mut filters: Vec<String> = Vec::new();
        for raw in raw_filters {
            let rfilter = Regex::new(r"(\w*)(!=|>=|<=|>|<|=|~)'(.*)'").unwrap();
            for cap in rfilter.captures_iter(raw) {
                let mut predicate = &cap[2];
                if predicate == "=" {
                    predicate = "==";
                };
                let filter = format!(".{} {} \"{}\"", &cap[1], predicate, &cap[3].trim());
                log::debug!("filter: {}", filter);
                filters.push(filter);
            }
        }

        let mut filter_builder: Vec<String> = Vec::new();
        filter_builder.push(filters.pop().unwrap());
        for _ in 0..logicals.len() {
            filter_builder.push(logicals.pop().unwrap());
            filter_builder.push(filters.pop().unwrap());
        }

        return Some(filter_builder.join(" "));
    }
    None
}

async fn parse_fields(params: &Parameters) -> Option<String> {
    if let Some(q_field) = &params.fields {
        let mut fields = Vec::new();
        match q_field.contains(',') {
            true => {
                for f in q_field.split(',') {
                    fields.push(format!("{}: .{}", f, f));
                }
            }
            false => fields.push(format!("{}: .{}", q_field, q_field)),
        }
        return Some(fields.join(","));
    }
    None
}
pub(crate) async fn parse_query(req: &tide::Request<State>) -> Result<Parameters> {
    //let mut params = Parameters::default();
    let params = req.query().unwrap();
    println!("{:?}", params);
    Ok(params)
}

/*
pub async fn parse_sort(
    sort: &str,
    mut rows: Vec<crate::model::AcademicSession>,
) -> Result<Vec<crate::model::AcademicSession>> {
    let mut invalid = false;
    rows.sort_by(|a, b| match sort {
        "sourcedId" => a.sourced_id.cmp(&b.sourced_id),
        "status" => a.status.cmp(&b.status),
        "year" => a.year.cmp(&b.year),
        _ => {
            invalid = true;
            a.sourced_id.cmp(&b.sourced_id)
        }
    });
    if invalid {
        return Err(crate::server::ServerError::NoRecordDeleted);
    }
    Ok(rows)
}
*/
