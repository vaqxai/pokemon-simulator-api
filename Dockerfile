FROM rustlang/rust:nightly
COPY ./ ./
RUN cargo build --release
ENV RUST_LOG=debug
EXPOSE 8000
VOLUME /config
ENTRYPOINT ["./target/release/pokemon-simulator"]