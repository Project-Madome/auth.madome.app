FROM ubuntu:focal

ARG BINARY_FILE

COPY $BINARY_FILE /madome-auth
# COPY ./.env.release /.env

EXPOSE 3112

ENTRYPOINT [ "/madome-auth" ]
