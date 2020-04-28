# ibc-demo

```
$ git clone https://github.com/cdot-network/ibc-demo.git
$ submodule update --init
$ cd node-template
$ cargo build --release
$ ./target/release/node-template --base-path /tmp/chain-appia --dev
$ ./target/release/node-template --base-path /tmp/chain-flaminia --dev --port 20333 --ws-port 8844
$ ./target/release/cli appia create-client flaminia
$ ./target/release/cli flaminia create-client appia
$ ./target/release/cli appia bind-port bank
$ ./target/release/cli flaminia bind-port bank
$ ./target/release/cli appia release-port bank // don't
$ export RUST_LOG=relayer=info
$ ./target/release/relayer -c relayer/config.toml
$ ./target/release/cli appia conn-open-init 53a954d6a7b1c595e025226e5f2a1782fdea30cd8b0d207ed4cdb040af3bfa10 779ca65108d1d515c3e4bc2e9f6d2f90e27b33b147864d1cd422d9f92ce08e03
$ ./target/release/cli appia chan-open-init d93fc49e1b2087234a1e2fc204b500da5d16874e631e761bdab932b37907bd11 bank bank
$ ./target/release/cli appia send-packet 1 1000 bank 00e2e14470ed9a017f586dfe6b76bb0871a8c91c3151778de110db3dfcc286ac bank a1611bcd0ba368e921b1bd3eb4aa66534429b14837725e8cef28182c25db601e 01020304
```
