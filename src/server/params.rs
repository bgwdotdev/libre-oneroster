use super::{Result, ServerError, State};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
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

pub(crate) async fn apply_parameters(
    json: &String,
    params: &Parameters,
    wrapper: &str,
) -> Result<(String, String)> {
    let (query, length) = parameter_builder(&params, wrapper).await?;
    let output = jq_rs::run(&query, &json).map_err(|e| {
        log::debug!("{}", json);
        log::debug!("jq error: {}: query: {}", e, query);
        ServerError::InvalidParameters
    })?;
    let total = jq_rs::run(&length, &json).map_err(|_| ServerError::InvalidParameters)?;
    Ok((output, total))
}

async fn parameter_builder(params: &Parameters, wrapper: &str) -> Result<(String, String)> {
    let mut builder = format!("{{ {}: [.{}[] ", wrapper, wrapper);

    if let Some(fields) = parse_fields(params).await {
        let f = format!("| {{ {} }} ", fields);
        builder.push_str(&f);
    }

    if let Some(filter) = parse_filter(params).await? {
        let f = format!("| select({}) ", filter);
        builder.push_str(&f);
    }

    builder.push_str("] ");
    let mut len: String = builder.clone();
    len.push_str(&format!("| length }} | .{}", wrapper));

    let pagination = format!("| .[{}:{}]", params.offset, params.offset + params.limit);
    builder.push_str(&pagination);

    if let Some(sort) = parse_sort(params).await {
        let f = format!("| sort_by(.{}) ", sort);
        builder.push_str(&f);
    }
    builder.push_str("} ");

    log::debug!("parameter jq builder: {}", builder);
    Ok((builder, len))
}

async fn parse_sort(params: &Parameters) -> Option<String> {
    if let Some(q_sort) = &params.sort {
        return Some(q_sort.to_owned());
    }
    None
}

async fn parse_filter(params: &Parameters) -> Result<Option<String>> {
    if let Some(q_filter) = &params.filter {
        let rlogic = Regex::new(r" (AND|OR) ")?;
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
            let rfilter = Regex::new(r"(\w*)(!=|>=|<=|>|<|=|~)'(.*)'")?;
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
        filter_builder.push(filters.pop().ok_or(ServerError::InvalidFilterField)?);
        for _ in 0..logicals.len() {
            filter_builder.push(logicals.pop().ok_or(ServerError::InvalidFilterField)?);
            filter_builder.push(filters.pop().ok_or(ServerError::InvalidFilterField)?);
        }

        return Ok(Some(filter_builder.join(" ")));
    }
    Ok(None)
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

// TODO: review url building, issue with .path() not returning sub router prefix
pub(super) async fn link_header_builder(
    req: &tide::Request<State>,
    params: &Parameters,
    data_len: usize,
) -> String {
    println!("{:?}", req.url());
    let mut link = String::from("");
    if params.limit <= data_len as u32 {
        link = format!(
            "<{}://{}:{}/ims/oneroster/v1p1{}?offset={}&limit={}>; rel=\"next\",",
            req.url().scheme(),
            req.url().domain().unwrap_or("localhost"),
            req.url().port().unwrap_or(443),
            req.url().path(),
            params.offset + params.limit,
            params.limit
        )
    };
    link
}
