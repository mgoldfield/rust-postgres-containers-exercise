FROM rust:1.69 as builder
WORKDIR /builder/
COPY ./benchmark-queries/ .

# in the future could cache dependencies for faster build
RUN cargo install --path .; 

FROM debian:stable-slim
COPY --from=builder /usr/local/cargo/bin/benchmark-queries /usr/local/bin/benchmark-queries
CMD ["benchmark-queries"]