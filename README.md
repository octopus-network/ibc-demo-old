# ibc-demo

## Build

```bash
$ cargo build --release
$ cargo build -p relayer --release
```

## Run

### start two dev chains

```bash
$ ./target/release/ibc-node --base-path /tmp/chain-a --port 30333 --ws-port 9944 --dev
$ ./target/release/ibc-node --base-path /tmp/chain-b --port 30334 --ws-port 9945 --dev
```

### start a relayer process

```bash
$ ./target/release/relayer run --addr1 127.0.0.1:9944 --addr2 127.0.0.1:9945
```

### send an interchain message to chain A

```bash
$ ./target/release/relayer interchain-message --message 01020304 --nonce 0 --para-id 0
```
