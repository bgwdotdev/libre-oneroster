use async_std::task;
use clap;
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
        .subcommand(
            clap::App::new("server")
                .arg(
                    clap::Arg::new("socket_address")
                        .about("address to bind server to")
                        .short('a')
                        .long("address")
                        .takes_value(true)
                        .value_name("IP:PORT")
                        .default_value("127.0.0.1:8080"),
                )
                .arg(
                    clap::Arg::new("init")
                        .about("initializes the database and provides admin credentials")
                        .long("init")
                        .takes_value(false),
                )
                .arg(
                    clap::Arg::new("database")
                        .about("Path to the database file")
                        .short('d')
                        .long("database")
                        .takes_value(true)
                        .value_name("PATH")
                        .default_value("oneroster.db"),
                )
                .arg(
                    clap::Arg::new("private_key")
                        .about("path to the pem encoded private key used to encode the JWT")
                        .long("private-key")
                        .takes_value(true)
                        .value_name("PATH")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("public_key")
                        .about("path to the pem encoded public key used to decode the JWT")
                        .long("public-key")
                        .takes_value(true)
                        .value_name("PATH")
                        .required(true),
                ),
        )
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
        Some(("server", args)) => {
            let c = server::Config {
                database: args.value_of_t("database").unwrap(),
                init: args.is_present("init"),
                socket_address: args.value_of_t("socket_address").unwrap(),
                encode_key: server::read_private_key(args.value_of("private_key").unwrap())
                    .unwrap(),
                decode_key: server::read_public_key(args.value_of("public_key").unwrap()).unwrap(),
            };
            task::block_on(server::run(c)).unwrap();
        }
        _ => {}
    }
}
