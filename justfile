check:
  cd consumer-side &&  cargo check --target wasm32-unknown-unknown --lib
  cd ibc-hooks-contract && cargo check --lib

clippy:
  cd consumer-side && cargo +nightly clippy --tests
  cd ibc-hooks-contract && cargo +nightly clippy --tests

test:
  cd consumer-side && cargo test
  cd ibc-hooks-contract && cargo test

fmt:
  cd consumer-side && cargo +nightly fmt
  cd ibc-hooks-contract && cargo +nightly fmt


optimize:
    just optimize-consumer
    just optimize-hooks

optimize-consumer:
  if [[ $(uname -m) =~ "arm64" ]]; then \
    just optimize-consumer-arm; else \
    just optimize-consumer-x86; fi

optimize-consumer-arm:
  cd consumer-side && \
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    --platform linux/arm64 \
    cosmwasm/rust-optimizer-arm64:0.15.1

optimize-consumer-x86:
  cd consumer-side && \
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    --platform linux/amd64 \
    cosmwasm/rust-optimizer:0.15.1

optimize-hooks:
    cd ibc-hooks-contract && \
    docker run --rm -v "$(pwd)":/contract \
        --mount type=volume,source="$(basename "$(pwd)")_cache",target=/contract/target \
        --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
        enigmampc/secret-contract-optimizer:1.0.10
