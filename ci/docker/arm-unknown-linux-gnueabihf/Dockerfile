FROM ubuntu:18.04

RUN apt-get update -y && apt-get install -y --no-install-recommends \
  ca-certificates \
  make \
  perl \
  gcc \
  libc6-dev \
  gcc-arm-linux-gnueabihf \
  gcc-multilib-arm-linux-gnueabihf \
  libatomic1-armhf-cross \
  libc6-dev-armhf-armel-cross

ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_RUNNER=echo \
  CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
