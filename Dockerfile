# Stage 1: Build Frontend
FROM node:20-slim as frontend-builder

WORKDIR /usr/src/frontend
COPY package.json package-lock.json ./
RUN npm ci

# Copy only necessary frontend files
COPY tsconfig.json tsconfig.app.json tsconfig.node.json vite.config.ts index.html tailwind.config.js postcss.config.js ./
COPY public ./public
COPY src ./src

RUN npm run build

# Stage 2: Build Backend
FROM rust:1.84-slim-bookworm as backend-builder

WORKDIR /usr/src/backend

# Copy manifests first to cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock ./

# Create a dummy project to build dependencies.
# We do this instead of just `cargo fetch` because we want to cache the 
# compiled results of the dependencies (which is the slowest part of the build), 
# not just the downloaded source code.
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Remove dummy source
RUN rm -rf src

# Copy real source
COPY backend/src ./src

# Build the application
# Use `touch` to ensure main.rs is newer than the cached build artifact
RUN touch src/main.rs && cargo build --release

# Stage 3: Runtime
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /usr/src/backend/target/release/backend /usr/local/bin/backend

# Copy frontend static files
COPY --from=frontend-builder /usr/src/frontend/dist ./dist

EXPOSE 3000

CMD ["backend"]
