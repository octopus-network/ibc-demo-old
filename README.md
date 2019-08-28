# ibc-demo

## Build

```bash
$ cargo build --release
$ cargo build -p relayer --release
```

## Run

### start a dev chain

```bash
$ ./target/release/ibc-node --dev
```

### start a relayer process

```bash
$ ./target/release/relayer start
```

### send an interchain message

```bash
$ ./target/release/relayer interchain-message --message 01020304 --nonce 0 --para-id 0
```
