FROM rust:1.49 as builder

RUN USER=root cargo new --bin voodoo-doll
WORKDIR ./voodoo-doll
COPY ./Cargo.toml ./Cargo.toml
RUN touch src/lib.rs
RUN cargo build --release --bin voodoo-doll
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/voodoo_doll*
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y libssl-dev

EXPOSE 80

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /voodoo-doll/target/release/voodoo-doll ${APP}/voodoo-doll

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./voodoo-doll"]
