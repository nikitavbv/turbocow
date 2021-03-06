FROM frolvlad/alpine-glibc:alpine-3.13_glibc-2.33
WORKDIR /app
ARG CRATE_NAME

RUN apk add libxkbcommon

COPY $CRATE_NAME/target/release/$CRATE_NAME /app/app

ENTRYPOINT [ "/app/app" ]
