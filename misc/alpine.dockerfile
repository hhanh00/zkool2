FROM rust:alpine AS builder

WORKDIR /zkool
COPY . .
ENV RUSTFLAGS='--cfg zcash_unstable="nu7"'
RUN apk add --no-cache perl build-base eudev-dev pkgconf
RUN cd rust && cargo build --release --bin zkool_graphql \
    --no-default-features --features=graphql,bundled-sapling-params,ledger

FROM alpine
RUN apk add --no-cache eudev-libs
COPY --from=builder /zkool/target/release/zkool_graphql /bin/zkool_graphql
ENTRYPOINT [ "/bin/zkool_graphql" ]
