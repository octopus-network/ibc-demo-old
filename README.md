# ibc-demo

```
$ cd node-template
$ cargo build --release
$ cargo build --release -p relayer
$ cargo build --release -p cli
$ ./target/release/node-template --base-path /tmp/chain-appia --dev -lruntime=debug --execution Native
$ ./target/release/node-template --base-path /tmp/chain-flaminia --port 20333 --ws-port 8844 --grafana-port 8855 --dev -lruntime=debug --execution Native
$ ./target/release/cli create-client 127.0.0.1:9944 flaminia
identifier: 0x779ca65108d1d515c3e4bc2e9f6d2f90e27b33b147864d1cd422d9f92ce08e03
$ ./target/release/cli create-client 127.0.0.1:8844 appia
identifier: 0x53a954d6a7b1c595e025226e5f2a1782fdea30cd8b0d207ed4cdb040af3bfa10
$ export RUST_LOG=info
$ ./target/release/relayer run 127.0.0.1:9944 127.0.0.1:8844
$ ./target/release/cli conn-open-init 127.0.0.1:9944 779ca65108d1d515c3e4bc2e9f6d2f90e27b33b147864d1cd422d9f92ce08e03 53a954d6a7b1c595e025226e5f2a1782fdea30cd8b0d207ed4cdb040af3bfa10
$ ./target/release/cli bind-port 127.0.0.1:9944 bank
$ ./target/release/cli bind-port 127.0.0.1:8844 bank
$ ./target/release/cli release-port 127.0.0.1:9944 bank // don't
$ ./target/release/cli chan-open-init 127.0.0.1:9944 8f97a7b961a7d4f26881763cc0b8507d2974d4cdda34e232a24a9f476d006f41 bank bank
$ ./target/release/cli send-packet 127.0.0.1:9944 1 1000 bank 3ba6953490756dfe8a6926d55f9e732f48be42c2559494db2e79c4df77bf6223 bank 7f18c08575c07b2e83408bc9122dfe613b386d82a908b937179db7e0d628bd38 01020304
```
