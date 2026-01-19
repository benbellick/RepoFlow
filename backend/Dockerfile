# Stage 1: Build
FROM rust:1.83-slim-bookworm as builder

WORKDIR /usr/src/app

# Copy manifests first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy project to build dependencies.
# We do this instead of just `cargo fetch` because we want to cache the 
# compiled results of the dependencies (which is the slowest part of the build), 
# not just the downloaded source code.
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Remove dummy source
RUN rm -rf src

# Copy real source
COPY src ./src

# Build the application
# Use `touch` to ensure main.rs is newer than the cached build artifact
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/backend

EXPOSE 3000

CMD ["backend"]
