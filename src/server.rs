use tide::prelude::*;
use tide::Request;

pub async fn run() -> tide::Result<()> {
    let mut srv = tide::new();
    srv.at("/academicSessions").get(get_all_academic_sessions);
    srv.listen("localhost:8080").await?;
    Ok(())
}

async fn get_all_academic_sessions(_req: Request<()>) -> tide::Result<String> {
    Ok("hi \n".to_string())
}
