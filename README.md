# move-language-server
Implementation of Language Server protocol for Move language

```shell script
RUST_LOG=info cargo run --bin move-language-server
```

## Configuration

`dialect` - dialect of the Move language. Either `move` (for original Libra version) or `dfinance` (bech32 addresses and some other stuff). Default is `move`.

`sender_address` - address of the user, used for module imports. Default is `0x0`.

`modules_folders` - array of folder paths for module lookup. Default is empty array.
