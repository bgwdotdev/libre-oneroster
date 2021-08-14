pub mod isams;
pub mod wcbs_pass;

use crate::{client, model};
use async_std::net::TcpStream;
use std::fmt;
use std::str::FromStr;
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
    pub provider: Provider,
}

pub enum Provider {
    Isams,
    WcbsPass,
}

impl FromStr for Provider {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "isams" => Ok(Provider::Isams),
            "pass" => Ok(Provider::WcbsPass),
            _ => Err("no match"),
        }
    }
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

async fn sync3<T>(config: &mut SyncConf, endpoint: &str, query: &str) -> Result<()>
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
    let mut datas: Vec<String> = Vec::new();
    for row in rows {
        if let Some(data) = row.try_get::<&str, _>(endpoint)? {
            datas.push(data.to_owned());
        }
    }
    let json = datas.join(",");
    let wrapped_json = format!(r#"{{"{}":[{}]}}"#, endpoint, json);
    let output: T = serde_json::from_str(&wrapped_json)?;
    client::put_all(&config.oneroster, &config.token, output, endpoint).await?;
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

    match &config.provider {
        Provider::Isams => {
            sync3::<model::AcademicSessions>(
                &mut sync_conf,
                "academicSessions",
                isams::QUERY_ACADEMIC_SESSIONS,
            )
            .await?;
            sync3::<model::Orgs>(&mut sync_conf, "orgs", isams::QUERY_ORGS).await?;
            sync3::<model::Courses>(&mut sync_conf, "courses", isams::QUERY_COURSES).await?;
            sync3::<model::Classes>(&mut sync_conf, "classes", isams::QUERY_CLASSES).await?;
            sync3::<model::Users>(&mut sync_conf, "users", isams::QUERY_USERS).await?;
            sync3::<model::Enrollments>(&mut sync_conf, "enrollments", isams::QUERY_ENROLLMENTS)
                .await?;
            Ok(())
        }
        Provider::WcbsPass => {
            sync2::<model::AcademicSessions>(
                &mut sync_conf,
                "academicSessions",
                wcbs_pass::QUERY_ACADEMIC_SESSIONS,
            )
            .await?;
            sync2::<model::Orgs>(&mut sync_conf, "orgs", wcbs_pass::QUERY_ORGS).await?;
            sync2::<model::Subjects>(&mut sync_conf, "subjects", wcbs_pass::QUERY_SUBJECTS).await?;
            sync2::<model::Periods>(&mut sync_conf, "periods", wcbs_pass::QUERY_PERIODS).await?;
            sync2::<model::Courses>(&mut sync_conf, "courses", wcbs_pass::QUERY_COURSES).await?;
            sync2::<model::Classes>(&mut sync_conf, "classes", wcbs_pass::QUERY_CLASSES).await?;
            sync2::<model::Users>(&mut sync_conf, "users", wcbs_pass::QUERY_USERS).await?;
            sync2::<model::Enrollments>(
                &mut sync_conf,
                "enrollments",
                wcbs_pass::QUERY_ENROLLMENTS,
            )
            .await?;
            Ok(())
        }
    }
}
