FROM rust:alpine AS builder

WORKDIR /zkool
COPY . .
RUN apk add perl build-base
RUN cd rust && cargo b --release --features=graphql --bin zkool_graphql

FROM alpine
COPY --from=builder /zkool/target/release/zkool_graphql /bin/zkool_graphql
ENTRYPOINT [ "/bin/zkool_graphql" ]
