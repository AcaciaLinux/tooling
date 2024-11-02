FROM debian AS build
USER root
WORKDIR /root

RUN apt update && apt install -y curl gcc
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup
RUN sh ./rustup -y

RUN ls

RUN apt install -y libssl-dev perl make git pkg-config

COPY tooling-codegen tooling-codegen
COPY .git .git
COPY src src
COPY build.rs .
COPY Cargo.toml .
COPY Cargo.lock .

RUN ~/.cargo/bin/cargo build --release --features trunk

FROM debian

RUN apt-get update && apt-get install -y openssl

COPY --from=build /root/target/release/trunk /bin/trunk

ENTRYPOINT [ "/bin/trunk" , "-v", "1",  "--home" , "/acacia"]

EXPOSE 27015/tcp
