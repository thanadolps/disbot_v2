FROM rustlang/rust:nightly-slim AS chef
RUN rustup toolchain uninstall nightly && \
    rustup toolchain install nightly-2025-04-03 --profile minimal --no-self-update && \
    rustup default nightly-2025-04-03 && \
    rustup component add rustc-codegen-cranelift-preview --toolchain nightly-2025-04-03
RUN cargo install cargo-chef
WORKDIR /app


FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json


FROM chef AS builder
COPY .cargo /app/.cargo
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    mold \
    m4 \
    make \
    && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin disbot_v2

FROM python:3-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/disbot_v2 ./disbot_v2
COPY ./python_dir ./python_dir
CMD ["./disbot_v2"]