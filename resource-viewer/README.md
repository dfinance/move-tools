# Move Resource Viewer

Console tool that gets resources from chain (by [dnode][]) and represents them in json or human-readable format.
Output describes entire structure, contains actual values.


## What does it do?

1. Makes request to database client (node) for specified query (address + resource type)
2. Restores full type layout of the requested resource asking node
3. Returns full type layout annotated with values which is actual for current state.


## Installation

Prerequisites:
- [Rust][] toolchain, the easiest way to get it is to use [Rustup][]

To install using cargo run the following commands:

```bash
cargo install --git https://github.com/dfinance/move-tools.git move-resource-viewer
```


[Rust]: https://www.rust-lang.org
[Rustup]: https://rustup.rs



## Usage example

```bash
move-resource-viewer -a wallet1n9w22mvaq7uuswr4j53usd0spd2mznphq3q3zp \
                          -q "0x1::Account::Balance<0x1::ETH::T>" \
                          --api="[https://rest.demo2.dfinance.co](https://rest.demo2.dfinance.co/)" \
                          -o=output.json
# optional block number:  --height 42

# Optionally add          --json-schema schema.json
# or just                 --json-schema -
# It exports schema for output format to specified file (schema.json)
# In case of `-` as path, it just prints schema to stdout.
```

### Input parameters

- `-a` / `--account` can be in DFinance [bech32][] or hex `0x…{16-20 bytes}` encoding formats
- `-q` / `--query` resource type-path, e.g.:
    - `0x1::Account::Balance<0x1::XFI::T>`
    - `0x1::Account::Balance<0x1::Coins::ETH>`
    - In general: `0xDEADBEEF::Module::Struct< 0xBADBEEF::Mod::Struct<...>, ... >`
    - Inner address can be omitted, it is inherited by parent:
	   `0xDEADBEEF::Module::Struct<Mod::Struct>` expands to `0xDEADBEEF::Module::Struct<0xDEADBEEF::Mod::Struct>`
    - Query can ends with index `[42]` for `vec`-resources
- Output options:
    - `-o` / `--output` fs-path to output file
    - `-j` / `--json` sets output format to json. Can be omitted if output file extension is `.json`, so then json format will be chosen automatically.
    - `--json-schema` additional json-schema export, fs-path to output schema file.

For more info check out `--help`.



[dnode]: https://github.com/dfinance/dnode
[bech32]: https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki



### Output

Two output formats supported:

- Move-like text
- JSON

_Structure of the output in JSON is described in the scheme, which can be obtained by calling with the `--json-schema` parameter._

#### `Move`-like example:

```rust
resource 00000000::Account::Balance<00000000::Coins::BTC> {
    coin: resource 00000000::Dfinance::T<00000000::Coins::BTC> {
        value: 1000000000u128
    }
}
```

#### `JSON` example:

<details>
  <summary>full json output</summary>

```json
{
  "is_resource": true,
  "type": {
    "address": "0000000000000000000000000000000000000001",
    "module": "Account",
    "name": "Balance",
    "type_params": [
      {
        "Struct": {
          "address": "0000000000000000000000000000000000000001",
          "module": "Coins",
          "name": "BTC",
          "type_params": []
        }
      }
    ]
  },
  "value": [
    {
      "id": "coin",
      "value": {
        "Struct": {
          "is_resource": true,
          "type": {
            "address": "0000000000000000000000000000000000000001",
            "module": "Dfinance",
            "name": "T",
            "type_params": [
              {
                "Struct": {
                  "address": "0000000000000000000000000000000000000001",
                  "module": "Coins",
                  "name": "BTC",
                  "type_params": []
                }
              }
            ]
          },
          "value": [
            {
              "id": "value",
              "value": {
                "U128": 1000000000
              }
            }
          ]
        }
      }
    }
  ]
}
```

</details>
