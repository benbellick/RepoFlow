# Stage 1: Build Frontend
FROM node:20-slim as frontend-builder

WORKDIR /usr/src/frontend
# Copy manifests first to cache dependencies
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

# Copy only necessary frontend files
COPY frontend/tsconfig.json frontend/tsconfig.app.json frontend/tsconfig.node.json frontend/vite.config.ts frontend/index.html frontend/tailwind.config.js frontend/postcss.config.js ./
COPY frontend/public ./public
COPY frontend/src ./src

RUN npm run build

# Stage 2: Build Backend
FROM rust:1.84-slim-bookworm as backend-builder

WORKDIR /usr/src/backend

# Copy manifests first to cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock ./

# Create a dummy project to build dependencies.
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Remove dummy source
RUN rm -rf src

# Copy real source
COPY backend/src ./src

# Build the application
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