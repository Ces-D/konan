FROM rust:1.91-alpine AS builder

WORKDIR /app
RUN apk add --no-cache musl-dev linux-headers libusb-dev eudev-dev
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/static-debian12 AS runner
EXPOSE 8080
COPY --from=builder /app/target/release/server /

CMD ["./server"]
