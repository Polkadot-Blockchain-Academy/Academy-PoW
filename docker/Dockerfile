FROM ubuntu:jammy-20220531

RUN apt update && \
        apt install curl -y && \
        apt clean && \
        rm -rf /var/lib/apt/lists/*

EXPOSE 30333 30343 9933 9944

WORKDIR node

# COPY target/release/academy-pow-node /usr/local/bin
COPY academy-pow/target/release/academy-pow /usr/local/bin/academy-pow-node
RUN chmod +x /usr/local/bin/academy-pow-node

COPY academy-pow/docker/docker_entrypoint.sh /node/docker_entrypoint.sh
RUN chmod +x /node/docker_entrypoint.sh

ENTRYPOINT ["./docker_entrypoint.sh"]
