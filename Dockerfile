FROM docker.io/library/rust:1.76 AS builder
WORKDIR /app
COPY . .
RUN  cargo build --release

# refer to: issue #12
FROM gcr.io/distroless/cc
WORKDIR /app
COPY --from=builder /app/target/release/interrupt-callback ./ic
EXPOSE 9080
CMD ["./ic"]