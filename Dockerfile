FROM rust:latest

WORKDIR /app

COPY . /app

EXPOSE 3000

RUN cargo build --release

# Run the application
CMD ["./target/release/collabweb"]