use async_std::task;
use clap;
use libre_oneroster::server::ServerError;
use libre_oneroster::{client, server};

fn main() {
    env_logger::init();
    match cli() {
        Ok(_) => (),
        Err(err) => match err {
            ServerError::Io(ref e) => log::error!("File error: {}", e),
            _ => println!("placeholder: {}", err),
        },
    }
}

fn cli() -> Result<(), ServerError> {
    let matches = clap::Command::new("libre-oneroster")
        .version("0.0.1")
        .subcommand_required(true)
        .subcommand(
            clap::Command::new("client")
                .arg(
                    clap::Arg::new("url")
                        .help("url to oneroster api")
                        .short('l')
                        .long("url")
                )
                .arg(
                    clap::Arg::new("clientid")
                        .help("client id for api auth")
                        .short('u')
                        .long("id")
                )
                .arg(
                    clap::Arg::new("clientsecret")
                        .help("client secret for api auth")
                        .short('p')
                        .env("OR_CS")
                        .long("secret")
                ),
        )
        .subcommand(
            clap::Command::new("sync")
                .about("Starts the MIS to oneroster sync client")
                .arg(
                    clap::Arg::new("database_ado_string")
                        .help("The source database ado connection string")
                        .short('d')
                        .long("database")
                        .value_name("ADO_STRING")
                        .required(true)
                        .long_help(
                            "tcp:ip\\instance;database=MyDatabase;\
                            username=MyUser;password=MySecret;\
                            encryption=true;TrustServerCertificate=true;",
                        ),
                )
                .arg(
                    clap::Arg::new("api")
                        .help("url to the oneroster API")
                        .short('u')
                        .long("url")
                        .value_name("URL")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("ci")
                        .help("client id to the oneroster API")
                        .short('i')
                        .long("client_id")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("cs")
                        .help("client secret to the oneroster API")
                        .short('p')
                        .long("client_secret")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("scope")
                        .help("oneroster scope required for calls")
                        .short('s')
                        .long("scope")
                        .default_value("roster-core.createput roster.createput"),
                )
                .arg(
                    clap::Arg::new("delta")
                        .help("The date/time of the last sync")
                        .short('t')
                        .long("delta")
                        .value_name("DATE_TIME")
                        .default_value("2015-01-01 00:00:00"),
                )
                .arg(
                    clap::Arg::new("year")
                        .help("The academic year to sync")
                        .short('y')
                        .long("year")
                        .value_name("YYYY")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("<provider>")
                        .help("The source database provider")
                        .value_parser(clap::builder::EnumValueParser::<crate::client::sync::Provider>::new())
                        .required(true),
                ),
        )
        .subcommand(
            clap::Command::new("server")
                .about("Starts the oneroster server")
                .arg(
                    clap::Arg::new("socket_address")
                        .help("address to bind server to")
                        .short('a')
                        .long("address")
                        .value_name("IP:PORT")
                        .value_parser(clap::value_parser!(std::net::SocketAddr))
                        .default_value("127.0.0.1:8080"),
                )
                .arg(
                    clap::Arg::new("init")
                        .help("initializes the database and provides admin credentials")
                        .long("init")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    clap::Arg::new("database")
                        .help("Path to the database file")
                        .short('d')
                        .long("database")
                        .value_name("PATH")
                        .default_value("oneroster.db"),
                )
                .arg(
                    clap::Arg::new("private_key")
                        .help("path to the pem encoded private key used to encode the JWT")
                        .short('J')
                        .long("private-key")
                        .value_name("PATH")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("public_key")
                        .help("path to the pem encoded public key used to decode the JWT")
                        .short('j')
                        .long("public-key")
                        .value_name("PATH")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("web_private_key")
                        .help("path to the pem encoded private key used to secure HTTPS")
                        .short('W')
                        .long("web-private-key")
                        .value_name("PATH")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("web_public_key")
                        .help("path to the pem encoded public key used to secure HTTPS")
                        .short('w')
                        .long("web-public-key")
                        .value_name("PATH")
                        .required(true),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("client", args)) => {
            let conf = client::Config {
                url: args.get_one::<String>("url").unwrap().to_string(),
                client_id: args.get_one::<String>("clientid").unwrap().to_string(),
                client_secret: args.get_one::<String>("clientsecret").unwrap().to_string(),
                scope: "admin.readonly".to_string(),
            };
            task::block_on(client::run(conf)).unwrap();
            Ok(())
        }
        Some(("server", args)) => {
            let encode_key = server::read_private_key(args.get_one::<String>("private_key").unwrap())
                .map_err(|e| {
                    log::error!("Problem reading private key");
                    e
                })?;
            let decode_key = server::read_public_key(args.get_one::<String>("public_key").unwrap())
                .map_err(|e| {
                    log::error!("Problem reading public key");
                    e
                })?;
            let c = server::Config {
                database: args.get_one::<String>("database").unwrap().to_string(),
                init: args.get_flag("init"),
                socket_address: *args.get_one("socket_address").unwrap(),
                encode_key,
                decode_key,
                web_public_key: args.get_one::<String>("web_public_key").unwrap().to_string(),
                web_private_key: args.get_one::<String>("web_private_key").unwrap().to_string(),
            };
            task::block_on(server::run(c)).unwrap();
            Ok(())
        }
        Some(("sync", args)) => {
            let or = crate::client::Config {
                url: args.get_one::<String>("api").unwrap().to_string(),
                client_id: args.get_one::<String>("ci").unwrap().to_string(),
                client_secret: args.get_one::<String>("cs").unwrap().to_string(),
                scope: args.get_one::<String>("scope").unwrap().to_string(),
            };
            let conf = crate::client::sync::Config {
                database_ado_string: args.get_one::<String>("database_ado_string").unwrap().to_string(),
                oneroster: or,
                delta: args.get_one::<String>("delta").unwrap().to_string(),
                academic_year: *args.get_one::<usize>("year").unwrap(),
                provider: *args.get_one("<provider>").unwrap(),
            };
            task::block_on(client::sync::sync(conf)).unwrap();
            Ok(())
        }
        _ => Ok(()),
    }
}
