FROM arm64v8/ubuntu:latest

ENV PATH="/flutter/bin:${PATH}:/root/.cargo/bin"
RUN apt-get update \
    && apt-get install -y build-essential clang cmake libssl-dev libcurl4-openssl-dev curl git unzip xz-utils zip ninja-build pkg-config libgtk-3-dev liblzma-dev libstdc++-12-dev \
    && apt-get clean \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && rustup default stable \
    && git clone https://github.com/flutter/flutter.git -b stable /flutter \
    && flutter doctor \
    && dart pub global activate fastforge
