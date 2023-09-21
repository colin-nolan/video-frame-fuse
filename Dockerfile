##################################################
# Builder
##################################################
FROM ubuntu:22.04 as builder

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
    | CARGO_HOME=/opt/cargo sh -s -- --default-toolchain 1.72.1 --profile default --no-modify-path -y
# Allow unknown users to cargo
RUN mkdir /.cargo && chmod 777 /.cargo

RUN git config --global --add safe.directory '*'


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

RUN cargo install --git https://github.com/kornelski/dssim.git --tag 3.1.0 --root /usr/

RUN apt-get update
RUN apt-get install -y --no-install-recommends \
        ffmpeg \
        imagemagick \
        jq \
        python-is-python3 \
        python3 \
        python3-pip \
        wget

# XXX: will break on non amd64, e.g. RPi
RUN wget https://github.com/mikefarah/yq/releases/download/v4.13.5/yq_linux_amd64 -O /usr/bin/yq \
    && chmod +x /usr/bin/yq

COPY tests/acceptance/scripts/image/requirements.txt /tmp/test-requirements.txt
RUN pip install -r /tmp/test-requirements.txt


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
# Extraction
##################################################
FROM scratch AS export

COPY --from=packager /usr/local/src/video-frame-fuse/target/release/video-frame-fuse /video-frame-fuse
