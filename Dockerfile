FROM ubuntu:20.04 as setup

SHELL ["/bin/bash", "-c"]

ENV DEBIAN_FRONTEND=noninteractive

# Note: not optimising layers in builder
RUN apt-get update
RUN apt-get install -y --no-install-recommends \
        ca-certificates \
        curl
RUN bash <(curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs) -y
ENV PATH "/root/.cargo/bin:${PATH}"

RUN apt-get install -y --no-install-recommends \
        llvm \
        libfuse-dev \
        libclang-dev \
        libopencv-dev \
        build-essential \
        clang

WORKDIR /usr/local/src/video-frame-fuse
ADD src/ ./src
ADD resources/ ./resources
ADD Cargo.* ./


FROM setup as builder

RUN cargo build --jobs $(nproc) --release


FROM setup as tester

RUN cargo test --jobs $(nproc)


FROM ubuntu:20.04 as production

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
        libopencv-dev \
        fuse \
   && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/src/video-frame-fuse/target/release/video-frame-fuse /usr/local/bin/

ENTRYPOINT ["video-frame-fuse"]
