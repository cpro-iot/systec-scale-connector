FROM ekidd/rust-musl-builder AS build
ADD --chown=rust:rust ./src ./src
ADD --chown=rust:rust ./Cargo.toml ./Cargo.toml
ADD --chown=rust:rust ./Cargo.lock ./Cargo.lock
RUN cargo build --release

FROM alpine:latest
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/scale-connector ./scale-connector

RUN chmod +x /scale-connector

ENV PORT=1234
ENV HOST=localhost
ENV INTERVAL=1000
ENV LOG=none
ENV MQTT=false

CMD /scale-connector ${HOST} -p ${PORT} -i ${INTERVAL} -l ${LOG} -m ${MQTT}
