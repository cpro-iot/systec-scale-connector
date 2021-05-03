FROM ekidd/rust-musl-builder AS build
#RUN sudo apt update && sudo apt upgrade -y && sudo apt install musl-tools gcc-multilib libssl-dev pkg-config openssl -y
RUN sudo apt update && sudo apt upgrade 
#&& sudo apt-get install llvm clang build-essential -y
RUN sudo apt-get install build-essential gcc make cmake cmake-gui cmake-curses-gui git doxygen graphviz libssl-dev llvm clang musl-tools -y
RUN git clone https://github.com/eclipse/paho.mqtt.c.git
WORKDIR /home/rust/src/paho.mqtt.c
RUN git checkout v1.3.8 
RUN cmake -Bbuild -H. -DPAHO_WITH_SSL=ON 
RUN sudo cmake --build build/ --target install 
RUN sudo ldconfig 
WORKDIR /home/rust/src
RUN git clone https://github.com/eclipse/paho.mqtt.cpp
WORKDIR /home/rust/src/paho.mqtt.cpp
RUN cmake -Bbuild -H. -DPAHO_BUILD_DOCUMENTATION=TRUE -DPAHO_BUILD_SAMPLES=TRUE
RUN sudo cmake --build build/ --target install
WORKDIR /home/rust/src

ADD --chown=rust:rust ./src ./src
ADD --chown=rust:rust ./Cargo.toml ./Cargo.toml
ADD --chown=rust:rust ./Cargo.lock ./Cargo.lock
RUN cargo build --release

FROM alpine:latest
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/scale-connector ./scale-connector
#COPY ./target/release/scale-connector /
RUN chmod +x /scale-connector
#RUN apk add bash
ENV PORT=1234
ENV HOST=localhost
ENV INTERVAL=1000
ENV LOG=none
ENV MQTT=none

#ENTRYPOINT [ "/bin/bash", "-l", "-c"]
#CMD ["/bin/bash", "-c", "/scale-connector ${HOST} -p ${PORT} -i ${INTERVAL} -l ${LOG}"]
#ENTRYPOINT [ "/scale-connector", "${HOST}", "-p ${PORT}", "-i ${INTERVAL}", "-l ${LOG}" ]
#ENTRYPOINT [ "/scale-connector", "${HOST}", "-p ${PORT}", "-l ${LOG}" ]
CMD /scale-connector ${HOST} -p ${PORT} -i ${INTERVAL} -l ${LOG} -m ${MQTT}
