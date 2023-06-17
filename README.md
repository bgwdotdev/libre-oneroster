# Libre OneRoster

A free, opensource and crossplatform oneroster 1.1 implementation in rust including:

    * client library and tools
    * server library and tools
    * CLI
    * webui?

With built-in export targets to:
    
    * Microsoft School Data Sync
    * Apple Schools Manager
    * Glow / RM-Unify

And from:
    
    * WCBS Pass
    * iSAMS

Backed by an json(SQLite) database.

## Build

### Container
Using or referencing the Dockerfile would be the best way to build. As there are a few C libraries in use, it cannot be built without sourcing shared libs.  
The main non-rust based dependencies being:
    * libjq -- for supporting being able to run any filters on any data endpoint
    * liboniguaram -- for jq regex support (to be removed?)
    * libopenssl -- for the certificate parsing for JWT generation

TODO: Add static linking build

## Server setup

### cli
```bash
oneroster server --help
# generate RSA x509 key pair for use with auth
# recommend using password and internal CA server
# Step used here for certificate generation but use your preferred option (openssl, certreq, etc)
step certificate create oneroster \
    oneroster.pem \
    oneroster.key.pem \
    --kty RSA \
    --insecure --no-password \
    --profile self-signed --subtle

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
podman run \
    --detach \
    --name oneroster \
    --port 8080:8080 \
    --volume $(pwd)/etc:/etc/opt/oneroster:z
    --volume $(pwd)/var:/var/opt/oneroster:z
    oneroster:1.0.0 \
        oneroster server \
        -d /var/opt/oneroster/oneroster.db \
        -j /etc/opt/oneroster/oneroster.pem \
        -J /etc/opt/oneroster/oneroster.key.pem \
        -w /etc/opt/oneroster/oneroster.pem \
        -W /etc/opt/oneroster/oneroster.key.pem
```

## Calling API from commandline

In this example, we are going to be using the following command line tools: `httpie` `jq`
```bash
# Get inital auth details from running oneroster server with --init flag
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


## Calling sync client with cli
```bash
# An SQL ADO connection string with your database information
database="server=tcp:192.168.100.100,1434;TrustServerCertificate=true;database=myDbInstance;username=onerosterService;password=aPassword;encrypt=true"

oneroster sync isams --database $isams --url $base --client_id $CI --client_secret $CS --scope roster-core.createput --year 2020
```

## TOOD: Calling API with oneroster cli
```bash
oneroster login

oneroster read academicSessions
oneroster put academicSessions @example.json

oneroster export SDS
oneroster export ASM
```
