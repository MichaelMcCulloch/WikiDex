FROM node:lts-slim AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build


FROM busybox:1.36.1

# Create a non-root user to own the files and run our server
RUN adduser -D static
USER static
WORKDIR /home/static
COPY --from=builder /app/build .

RUN echo "Ready on ${UI_CONT_PORT}"
CMD busybox httpd -f -v -p "${UI_CONT_PORT}"