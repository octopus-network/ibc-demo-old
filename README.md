# ibc-demo

```
$ cd node-template
$ cargo build --release
$ cargo build --release -p relayer
$ ./target/release/node-template --base-path /tmp/chain-appia --dev
$ ./target/release/node-template --base-path /tmp/chain-flaminia --port 20333 --ws-port 8844 --grafana-port 8855 --dev
$ ./target/release/cli create-client 127.0.0.1:9944 flaminia
identifier: 0x779ca65108d1d515c3e4bc2e9f6d2f90e27b33b147864d1cd422d9f92ce08e03
$ ./target/release/cli create-client 127.0.0.1:8844 appia
identifier: 0x53a954d6a7b1c595e025226e5f2a1782fdea30cd8b0d207ed4cdb040af3bfa10
$ ./target/release/relayer run 127.0.0.1:9944 127.0.0.1:8844
$ ./target/release/cli open-handshake 127.0.0.1:9944 779ca65108d1d515c3e4bc2e9f6d2f90e27b33b147864d1cd422d9f92ce08e03 53a954d6a7b1c595e025226e5f2a1782fdea30cd8b0d207ed4cdb040af3bfa10
```
