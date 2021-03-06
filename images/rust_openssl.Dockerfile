FROM ubuntu:focal

ENV TZ=Europe/Kiev
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update && apt-get install -y git make curl gcc pkg-config libssl-dev mingw-w64 build-essential libxkbcommon-x11-dev
RUN curl https://sh.rustup.rs -sSf > rustup.sh && chmod +x rustup.sh && bash rustup.sh -y
ENV PATH="$PATH:/root/.cargo/bin"

# install openssl
RUN git clone https://github.com/openssl/openssl.git
RUN cd openssl && \
    ./config && \
    make && make install && \
    mkdir lib && \
    cp *.so* lib && \
    cd ..

ENV OPENSSL_DIR=/openssl
ENV OPENSSL_STATIC=/openssl

RUN rustup install nightly

RUN rustup target add x86_64-pc-windows-gnu --toolchain nightly
