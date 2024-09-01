ARG PG_VERSION=16
FROM postgres:${PG_VERSION}
RUN apt-get update

ENV build_deps ca-certificates \
  git \
  build-essential \
  libpq-dev \
  postgresql-server-dev-${PG_MAJOR} \
  curl \
  libreadline6-dev \
  zlib1g-dev


RUN apt-get install -y --no-install-recommends $build_deps pkg-config cmake

WORKDIR /home/pgrx

ENV HOME=/home/pgrx \
  PATH=/home/pgrx/.cargo/bin:$PATH
RUN chown postgres:postgres /home/pgrx
USER postgres

RUN \
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain nightly && \
  rustup --version && \
  rustc --version && \
  cargo --version

# PGRX
RUN cargo install cargo-pgrx --version 0.11.4 --locked

RUN cargo pgrx init --pg${PG_MAJOR} $(which pg_config)

USER root

#COPY . .
#RUN cargo pgrx install

RUN chown postgres /usr/share/postgresql/${PG_MAJOR}/extension
RUN chown postgres /usr/lib/postgresql/${PG_MAJOR}/lib
RUN chown postgres /usr/lib/postgresql/${PG_MAJOR}/lib/bitcode

RUN apt-get install -y --no-install-recommends vim

USER postgres
