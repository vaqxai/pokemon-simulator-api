FROM rustlang/rust:nightly
COPY ./ ./
RUN cargo build --release
EXPOSE 8000
CMD ["./target/release/pokemon_simulator"]