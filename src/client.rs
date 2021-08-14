pub mod sync;

use crate::model;
use surf;

pub struct Config {
    pub url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
}

pub async fn run(conf: Config) -> surf::Result<()> {
    //env_logger::init();
    log::info!("run client..");
    let (client, token) = connect(conf).await?;
    get_all_academic_sessions(&client, &token).await?;

    Ok(())
}

pub async fn connect(conf: Config) -> surf::Result<(surf::Client, String)> {
    let mut client = surf::client();
    client.set_base_url(surf::Url::parse(conf.url.as_str())?);

    let token = login(&client, conf).await?;

    Ok((client, token))
}

async fn login(c: &surf::Client, conf: Config) -> surf::Result<String> {
    let cred = format!(
        "client_id={}&client_secret={}&scope={}",
        conf.client_id, conf.client_secret, conf.scope
    );
    log::debug!("{:?}", cred);
    let mut r = c
        .post("auth/login")
        .body(cred)
        .header("content-type", "application/x-www-form-urlencoded")
        .await?;
    log::debug!("response: {:?}", r);
    let t: model::TokenReturn = r.body_json().await?;
    let token = t.access_token;
    log::debug!("token={}", token);
    Ok(token)
}

async fn get_all_academic_sessions(c: &surf::Client, token: &String) -> surf::Result<()> {
    // doRequest("academicSessions")
    let t = format!("Bearer {}", token);
    let mut r = c
        .get("ims/oneroster/v1p1/academicSessions")
        .header("Authorization", t)
        .await?;
    log::debug!("{:?}", &r);
    println!("{}", r.body_string().await?);
    Ok(())
}

pub async fn put_all<T>(
    c: &surf::Client,
    token: &String,
    data: T,
    endpoint: &str,
) -> surf::Result<()>
where
    for<'a> T: serde::Serialize,
{
    c.put("ims/oneroster/v1p1/".to_owned() + endpoint)
        .body(serde_json::json!(data).to_string())
        .header("Authorization", "Bearer ".to_owned() + token)
        .await?;
    Ok(())
}

// async fn doRequest
