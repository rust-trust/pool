FROM rust:latest

WORKDIR app

RUN cargo install honggfuzz

COPY . .

# don't include [] or the command will be run directly rather than inside a shell - https://goinbigdata.com/docker-run-vs-cmd-vs-entrypoint/
CMD cd fuzz; cargo hfuzz run pool_fuzz