FROM ubuntu:18.04

RUN apt-get update -y && apt-get install -y --no-install-recommends \
  ca-certificates \
  make \
  perl \
  gcc \
  libc6-dev \
  musl-tools \
  linux-headers-4.15.0-20-generic

# link up the musl headers
RUN ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm \
  && ln -s /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic \
  && ln -s /usr/include/linux /usr/include/x86_64-linux-musl/linux
