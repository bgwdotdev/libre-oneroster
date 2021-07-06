# Libre OneRoster

A free, opensource and crossplatform oneroster 1.1 implementation in rust including:

    * client library and tools
    * server library and tools
    * CLI
    * webui?

With built-in export targets to:
    
    * Microsoft School Data Sync
    * Apple Schools Manager

And from:
    
    * WCBS Pass
    * iSAMS

Backed by an json(SQLite) database.

## Build 

depends on libjq and libonigurama to build

```bash
#suse
zypper install libjq-devel oniguruma-devel
source .env
cargo build 
```

## Server setup

```bash
# generate RSA x509 key pair for use with auth
# recommend using password and internal CA server
# step used here for certificate generation but use your preferred option (openssl, certreq, etc)
step certificate create oneroster \
    oneroster.pem \
    oneroster.key.pem \
    --kty RSA \
    --insecure --no-password \
    --profile self-signed --subtle

# sets up database and template config and provides one-time root creds
oneroster server --init
oneroster server --config oneroster.toml
```

## Calling API with traditional tools

```bash
# use preferred option (httpie, xh, curl, invoke-restmethod, etc)
oneroster="https://oneroster.example.com/ims/oneroster/v1p1"

# get bearer token using default root creds 
token=$(xh post $oneroster/login -f client_id=myId client_secret=mySecret scope="roster-core.readonly" | jq .access_token | xargs)

# create sample data and add
echo '{"sourcedId": 01, "status": "active"}' > example.json
xh put $oneroster/academicSessions Authorizaton:"Bearer $token" < example.json

# read sample data added
xh get $oneroster/academicSessions Authorizaton:"Bearer $token"
```

## Calling API with oneroster cli
```bash
oneroster login

oneroster read academicSessions
oneroster put academicSessions @example.json

oneroster export SDS
oneroster export ASM
```
