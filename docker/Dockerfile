FROM ubuntu:jammy-20220531

RUN apt update && \
        apt install curl -y && \
        apt clean && \
        rm -rf /var/lib/apt/lists/*

EXPOSE 30333 30343 9944

WORKDIR node

COPY target/release/academy-pow /usr/local/bin/academy-pow
RUN chmod +x /usr/local/bin/academy-pow

COPY docker/docker_entrypoint.sh /node/docker_entrypoint.sh
RUN chmod +x /node/docker_entrypoint.sh

ENTRYPOINT ["./docker_entrypoint.sh"]
