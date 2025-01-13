FROM rustlang/rust:nightly
COPY ./ ./
RUN cargo build --release
ENV RUST_LOG=debug
EXPOSE 8000
COPY ./Config.toml ./target/release/Config.toml
COPY ./Config.toml ./target/release/config.toml
CMD ["./target/release/pokemon_simulator"]