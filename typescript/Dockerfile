FROM node:lts-slim AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build


FROM busybox:1.36

# Create a non-root user to own the files and run our server
RUN adduser -D static
USER static
WORKDIR /home/static
COPY --from=builder /app/build .

EXPOSE "${HOST_API_PORT:-3000}:${CONTAINER_API_PORT:-3000}"
CMD busybox httpd -f -v -p "${CONTAINER_API_PORT:-3000}"