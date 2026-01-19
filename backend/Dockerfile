# Stage 1: Build
FROM rust:1.83-slim-bookworm as builder

WORKDIR /usr/src/app
COPY . .

# Build for release
RUN cargo build --release

# Stage 2: Runtime
# We use gcr.io/distroless/cc-debian12 for a tiny image that still has libc
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/backend

EXPOSE 3000

CMD ["backend"]
