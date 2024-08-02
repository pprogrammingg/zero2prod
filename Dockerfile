# We use the latest Rust stable release as base image
FROM rust:1.79.0
# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not # exist already.
WORKDIR /app
# Install the required system dependencies for our linking configuration
RUN apt update && apt install -y lld clang
# Copy all files from our working environment to our Docker image
COPY . .
# Let's build our binary!
# Use release profile for faster deployment
RUN cargo build --release
# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./target/release/zero2prod"]
