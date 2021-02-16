use async_std::task;
use libre_oneroster::{client, server};

fn main() {
    let matches = clap::App::new("libre-oneroster")
        .version("0.0.1")
        .subcommand(
            clap::App::new("client")
                .arg(
                    clap::Arg::new("url")
                        .about("url to oneroster api")
                        .short('l')
                        .long("url")
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::new("clientid")
                        .about("client id for api auth")
                        .short('u')
                        .long("id")
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::new("clientsecret")
                        .about("client secret for api auth")
                        .short('p')
                        .long("secret")
                        .env("OR_CS")
                        .takes_value(true),
                ),
        )
        .subcommand(clap::App::new("server"))
        .get_matches();

    match matches.subcommand() {
        Some(("client", cm)) => {
            let conf = client::Config {
                url: cm.value_of("url").unwrap().to_string(),
                client_id: cm.value_of("clientid").unwrap().to_string(),
                client_secret: cm.value_of("clientsecret").unwrap().to_string(),
            };
            task::block_on(client::run(conf)).unwrap();
        }
        Some(("server", _a)) => task::block_on(server::run()).unwrap(),
        _ => {}
    }
}
