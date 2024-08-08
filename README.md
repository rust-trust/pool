# Pool Program

Rust smart contract for Solana liquidity pools with variable token number.

## Building

To build the Pool program, use the normal build command for Solana programs:

```bash
cargo build-bpf
```

## Deployment

To deploy the pool program:

1. Check that the `TOKEN_COUNT` const is set to the number of constituent tokens you want the pool program to initialize
2. Build the program:

```bash
cargo build-bpf
```

3. Deploy the program using:

```bash
solana program deploy --program-id <path_to_keypair> ./target/deploy/pool.so
```

4. To adjust the number of constituent tokens for the Pool Program, adjust the `TOKEN_COUNT` const in `src/lib.rs` then rebuild and deploy the program to a new program_id

## Audits and Security

[Kudelski audit](https://swim.io/audits/kudelski.pdf) completed Dec 13th, 2021

## Mainnet Deployments

Pools with 4 Tokens: `SWiMBJS9iBU1rMLAKBVfp73ThW1xPPwKdBHEU2JFpuo`

Pools with 6 Tokens: `SWiMDJYFUGj6cPrQ6QYYYWZtvXQdRChSVAygDZDsCHC`

## Running Tests

```bash
cd swim/pool
# run all tests
cargo test-bpf -- --show-output
# run all tests with suppressed noisy logs
cargo test-bpf -- --show-output --nocapture --test-threads=1 2>&1 | ./sol_spam_filter.py
# run specific test
cargo test-bpf -- --test test_pool_init --show-output
```


# Scan for vulnerabilities
[Soteria](https://www.soteria.dev/post/soteria-a-vulnerability-scanner-for-solana-smart-contracts)
## Install
### On Linux
```bash
# install Soteria
cd ~
sh -c "$(curl -k https://supercompiler.xyz/install)"
# Depending on your system, you may need to change your PATH environment variable to include soteria
export PATH=$PWD/soteria-linux-develop/bin/:$PATH
```

### On Docker
```bash
docker run -v $PWD/:/workspace -it greencorelab/soteria:0.1.0 /bin/bash
```

## Check vulnerabilities
```bash
# check vulnerabilities in selected library codes
soteria .
# check vulnerabilities in all library codes
soteria -analyzeAll .
```

```sh
# 1. build docker container (only have to do this one time)
$ docker build -t pool .
# 2. run docker container
