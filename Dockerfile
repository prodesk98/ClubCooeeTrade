FROM ubuntu:24.10

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY ./target/release/ClubCooee /usr/local/bin/ClubCooee

ENTRYPOINT ["/usr/local/bin/ClubCooee"]
