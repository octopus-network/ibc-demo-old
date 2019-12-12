# ibc-demo

```
$ cd node-template
$ cargo build --release
$ cargo build --release -p relayer
$ ./target/release/node-template --base-path /tmp/chain-appia --dev
$ ./target/release/node-template --base-path /tmp/chain-flaminia --port 20333 --ws-port 8844 --grafana-port 8855 --dev
$ ./target/release/cli create-client 127.0.0.1:8844 appia
$ ./target/release/relayer run 127.0.0.1:9944 127.0.0.1:8844
```
