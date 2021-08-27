FROM registry.opensuse.org/opensuse/tumbleweed AS builder
# install deps
RUN zypper ref && zypper in -y \
    rust \
    cargo \
    gcc \
    # oniguruma-devel \
    libjq-devel \
    libopenssl-devel \
    sqlite3 \
    tar
# Create build dir
RUN mkdir --parents /opt/oneroster/build
WORKDIR /opt/oneroster/build
RUN mkdir src && \
    mkdir db && \
    echo "fn main(){}" > src/main.rs
# Set build vars
ENV JQ_NO_ONIG "true"
ENV JQ_LIB_DIR "/usr/lib64/libjq.so"
ENV DATABASE_URL "sqlite:db/oneroster.db"
# Build rust deps
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release
# Setup database reference for query verification
COPY db/schema.sql db/schema.sql
COPY db/init.sql db/init.sql
RUN sqlite3 db/oneroster.db < db/schema.sql && \
    sqlite3 db/oneroster.db < db/init.sql
#  Build oneroster
COPY src src
RUN cargo build --release
# package binary and libs into .tar
RUN mkdir --parents /opt/oneroster/bin && \
    cd /opt/oneroster && \
    cp build/target/release/oneroster bin/oneroster && \
    tar --create --file /opt/oneroster/build/oneroster.tar \
        bin/oneroster && \
    cd /usr && \
    tar --append --file /opt/oneroster/build/oneroster.tar \
        lib64/libjq.* \
        lib64/libcrypto.* \
        lib64/libssl.* \
        lib64/libgcc_s.* \
        lib64/libonig.*


FROM registry.opensuse.org/opensuse/busybox
WORKDIR /opt/oneroster
COPY --from=builder /opt/oneroster/build/oneroster.tar .
RUN tar x -f oneroster.tar && \
    rm oneroster.tar
ENV PATH "/opt/oneroster/bin:${PATH}"
ENV LD_LIBRARY_PATH "/opt/oneroster/lib64"
ENTRYPOINT [ "oneroster" ]
