FROM rust:1.88-bookworm

WORKDIR /app

RUN cargo install cargo-watch --locked

# Pre-cache dependencies by building a dummy project
COPY Cargo.toml Cargo.toml
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build 2>/dev/null || true
RUN rm -rf src

COPY . .

CMD ["cargo", "watch", "-x", "run"]
