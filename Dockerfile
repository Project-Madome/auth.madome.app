FROM ubuntu:focal

ARG BINARY_FILE

SHELL [ "/bin/bash", "-c" ]

RUN cat /etc/apt/sources.list | \
    sed -e "s/archive.ubuntu.com/mirror.kakao.com/g" | \
    sed -e "s/security.ubuntu.com/mirror.kakao.com/g" >> \
    /etc/apt/sources.list
RUN apt-get update && apt-get install -y ca-certificates

COPY $BINARY_FILE /madome-auth
# COPY ./.env.release /.env

EXPOSE 3112

ENTRYPOINT [ "/madome-auth" ]
