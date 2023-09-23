FROM rust:latest as builder
WORKDIR /api
COPY . .
RUN cargo build --release

FROM builder as runtime
COPY --from=builder /api/target/release/petompp-web-api .
COPY --from=builder /api/Rocket.toml .
EXPOSE 16969
CMD ["./petompp-web-api"]