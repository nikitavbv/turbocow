FROM frolvlad/alpine-glibc:alpine-3.12_glibc-2.32
WORKDIR /app
ARG CRATE_NAME

COPY $CRATE_NAME/target/release/$CRATE_NAME /app/app

ENTRYPOINT [ "/app/app" ]
