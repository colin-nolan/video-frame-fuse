##################################################
# Builder
##################################################
FROM ubuntu:20.04 as builder

SHELL ["/bin/bash", "-c"]

ENV DEBIAN_FRONTEND=noninteractive

# Note: not optimising layers in builder
RUN apt-get update
RUN apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        curl \
        git \
        libclang-dev \
        libfuse-dev \
        libopencv-dev \
        llvm

ENV RUSTUP_HOME=/opt/rustup
ENV PATH="${PATH}:/opt/cargo/bin"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | CARGO_HOME=/opt/cargo sh -s -- --default-toolchain stable --profile default --no-modify-path -y
# Allow unknown users to cargo
RUN mkdir /.cargo && chmod 777 /.cargo


##################################################
# Formatter
##################################################
FROM builder as formatter

ENTRYPOINT ["cargo", "fmt"]


##################################################
# Tester
##################################################
FROM builder as tester

RUN curl -fsSL https://git.io/shellspec | sh -s -- --prefix /usr/local --yes

RUN cargo install --git https://github.com/kornelski/dssim.git --tag 2.11.3 --root /usr/

RUN apt-get update
RUN apt-get install -y --no-install-recommends \
        ffmpeg


##################################################
# Package for production
##################################################
FROM builder as packager

WORKDIR /usr/local/src/video-frame-fuse
COPY scripts/build/run-release-build.sh .
COPY resources/ ./resources
COPY src/ ./src
COPY Cargo.* ./

RUN git init
RUN ./run-release-build.sh .


##################################################
# Production
##################################################
FROM ubuntu:20.04 as production

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
        libopencv-dev \
        fuse \
   && rm -rf /var/lib/apt/lists/*

COPY --from=packager /usr/local/src/video-frame-fuse/target/release/video-frame-fuse /usr/local/bin/

ENTRYPOINT ["video-frame-fuse"]
