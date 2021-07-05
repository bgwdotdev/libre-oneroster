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
    let matches = clap::App::new("libre-oneroster")
        .version("0.0.1")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
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
            clap::App::new("sync")
                .about("Starts the MIS to oneroster sync client")
                .arg(
                    clap::Arg::new("database_ado_string")
                        .about("The source database ado connection string")
                        .short('d')
                        .long("database")
                        .takes_value(true)
                        .value_name("ADO_STRING")
                        .required(true)
                        .long_about(
                            "tcp:ip\\instance;database=MyDatabase;\
                            username=MyUser;password=MySecret;\
                            encryption=true;TrustServerCertificate=true;",
                        ),
                )
                .arg(
                    clap::Arg::new("api")
                        .about("url to the oneroster API")
                        .short('u')
                        .long("url")
                        .takes_value(true)
                        .value_name("URL")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("ci")
                        .about("client id to the oneroster API")
                        .short('i')
                        .long("client_id")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    clap::Arg::new("cs")
                        .about("client secret to the oneroster API")
                        .short('p')
                        .long("client_secret")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    clap::Arg::new("scope")
                        .about("oneroster scope required for calls")
                        .short('s')
                        .long("scope")
                        .takes_value(true)
                        .default_value("roster-core.createput roster.createput"),
                )
                .arg(
                    clap::Arg::new("delta")
                        .about("The date/time of the last sync")
                        .short('t')
                        .long("delta")
                        .takes_value(true)
                        .value_name("DATE_TIME")
                        .default_value("2015-01-01 00:00:00"),
                )
                .arg(
                    clap::Arg::new("year")
                        .about("The academic year to sync")
                        .short('y')
                        .long("year")
                        .takes_value(true)
                        .value_name("YYYY")
                        .required(true),
                ),
        )
        .subcommand(
            clap::App::new("server")
                .about("Starts the oneroster server")
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
                scope: "admin.readonly".to_string(),
            };
            task::block_on(client::run(conf)).unwrap();
            Ok(())
        }
        Some(("server", args)) => {
            let encode_key = server::read_private_key(args.value_of("private_key").unwrap())
                .map_err(|e| {
                    log::error!("Problem reading private key");
                    e
                })?;
            let decode_key = server::read_public_key(args.value_of("public_key").unwrap())
                .map_err(|e| {
                    log::error!("Problem reading public key");
                    e
                })?;
            let c = server::Config {
                database: args.value_of_t("database").unwrap(),
                init: args.is_present("init"),
                socket_address: args.value_of_t("socket_address").unwrap(),
                encode_key,
                decode_key,
            };
            task::block_on(server::run(c)).unwrap();
            Ok(())
        }
        Some(("sync", args)) => {
            let or = crate::client::Config {
                url: args.value_of_t("api").unwrap(),
                client_id: args.value_of_t("ci").unwrap(),
                client_secret: args.value_of_t("cs").unwrap(),
                scope: args.value_of_t("scope").unwrap(),
            };
            let conf = crate::client::sync::wcbs_pass::Config {
                database_ado_string: args.value_of_t("database_ado_string").unwrap(),
                oneroster: or,
                delta: args.value_of_t("delta").unwrap(),
                academic_year: args.value_of_t("year").unwrap(),
            };
            task::block_on(client::sync::wcbs_pass::sync(conf)).unwrap();
            Ok(())
        }
        _ => Ok(()),
    }
}
