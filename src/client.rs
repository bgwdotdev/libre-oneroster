use surf;

pub struct Config {
    pub url: String,
    pub client_id: String,
    pub client_secret: String,
}

pub async fn run(conf: Config) -> surf::Result<()> {
    env_logger::init();
    log::info!("run client..");
    let mut client = surf::client();
    client.set_base_url(surf::Url::parse(conf.url.as_str())?);

    let t = login(&client, conf).await?;
    get_all_academic_sessions(&client, &t).await?;

    /*
    let mut r = surf::get("https://httpbin.org/get").await?;
    let out = r.body_string().await?;
    println!("{}", out);
    */

    Ok(())
}

async fn login(c: &surf::Client, conf: Config) -> surf::Result<String> {
    let cred = format!(
        "clientid={}&clientsecret={}",
        conf.client_id, conf.client_secret
    );
    let mut r = c
        .post("login")
        .body(cred)
        .header("content-type", "application/x-www-form-urlencoded")
        .await?;
    let t = r.body_string().await?;
    let token = t.trim().trim_matches('"').to_string();
    log::debug!("token={}", token);
    Ok(token)
}

async fn get_all_academic_sessions(c: &surf::Client, token: &String) -> surf::Result<()> {
    let t = format!("Bearer {}", token);
    let mut r = c.get("academicSessions").header("Authorization", t).await?;
    log::debug!("{:?}", &r);
    println!("{}", r.body_string().await?);
    Ok(())
}

