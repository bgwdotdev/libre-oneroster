# Libre OneRoster

A free, open-source and cross platform OneRoster 1.1 implementation in rust including:

* api server
* client library
* cli tools

With built-in export targets to:

* Microsoft School Data Sync
* Apple Schools Manager
* Glow / RM-Unify

And from:

* WCBS Pass
* iSAMS

An Sqlite database implements the majority of the read/write layer, supporting
reading and writing directly with oneroster formatted json. It should be
portable enough to use in other implementation with little work.


## Server setup

The server can initialise a new database with default admin credentials passing
the `--init` flag on start-up. These credentials will be printed to stdout and
never shown again so take a note of them.

The server uses two certificates, one for encrypting the web server HTTP
traffic, and one signing the json web token (JWT) used for the authentication
flow.

You can generate a self-signed certificate using `openssl` or small-steps
`step` cli, for example:

```bash
mkdir certs
step certificate create oneroster \
    certs/oneroster.crt \
    certs/oneroster.key \
    --kty RSA \
    --insecure \
    --no-password \
    --profile self-signed \
    --subtle
```

You can then either run the binary directly or using the docker compose reference provided.


### Cli

```bash
oneroster server --help
# sets up database and template config and provides one-time root creds
oneroster server \
    --init \
    --database myoneroster.db \
    --public-key oneroster.pem \
    --private-key oneroster.key.pem \
    --web-public-key oneroster.pem \
    --web-private-key oneroster.key.pem

# Can remove --init after database has been initialised for the first time
oneroster server -d myoneroster.db -j oneroster.pem -J oneroster.key.pem -w oneroster.pem -W oneroster.key.pem
```


### Container
```bash
docker compose up
```

### Logs

logging level can be configured via the `RUST_LOG` environment variable

```bash
RUST_LOG=debug oneroster server --init ...
```

## Client 

As a user/client of the server, you can call various web endpoints to retrieve
or upload OneRoster api compliant data to the server.

This can be done via common command line tools as shown below, via your own
OneRoster compliant code libraries and finally, using the input cli tools to,
for example, sync data directly from a supported MIS/SIS database or other
system.


### Calling API from command line

In this example, we are going to be using the following command line tools: `httpie` `jq`

```bash
# Get initial auth details from running oneroster server with --init flag
CI="myuser"
CS="mysecret"
scope="admin.readonly roster-core.readonly roster-core.createput"

token=$(https --verify false --form POST localhost:8080/auth/login client_id=$CI client_secret=$CS scope="$scope" | jq .access_token | xargs)

# define sample data
echo '{
  "academicSessions": [
    {
      "sourcedId": "01",
      "dateLastModified": "2021-01-01 00:00:00Z",
      "status": "active",
      "title": "exampleSession",
      "startDate": "2021-01-01",
      "endDate": "2021-02-01",
      "type": "term",
      "schoolYear": "2021"
    }
  ]
}' > example.json

# write data
https --verify false PUT localhost:8080/ims/oneroster/v1p1/academicSessions Authorization:"Bearer $token" < example.json

# read data
https --verify false GET localhost:8080/ims/oneroster/v1p1/academicSessions Authorization:"Bearer $token"
```


###  Calling sync client with cli

```bash
# An SQL ADO connection string with your database information
database="server=tcp:192.168.100.100,1434;TrustServerCertificate=true;database=myDbInstance;username=onerosterService;password=aPassword;encrypt=true"

oneroster sync isams \
    --database $database \
    --url $base \
    --client_id $CI \
    --client_secret $CS \
    --scope roster-core.createput \
    --year 2020
```


## Development

[Nix](https://nixos.org/) flakes are used for development and build tooling,
use the following command for a developer shell: `nix develop`

The application can also be build using `nix build` to create a normal build,
or `nix build .#docker` to build a docker image which can then be loaded with
`docker load -i ./result`


### Build

There there are a few C libraries in use, it cannot be built without sourcing
shared libs.  The main non-rust based dependencies being:

* libjq -- for supporting being able to run any filters on any data endpoint
* liboniguaram -- for jq regex support (to be removed?)
* libopenssl -- for the certificate parsing for JWT generation

On-top of the required C libraries, the rust tool-chain is required which can be
installed using [rustup](https://rustup.rs/).


### Make

The included `Makefile` provides common actions for building/developing the project.
