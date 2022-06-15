# mini-Paymens
## _The Last Markdown Editor, Ever_

mini-payments is a toy project simulating a payments service
which accepts and processes payment transactions such as
- Deposits
- Withdrawals
- Disputes
- Dispute Resolutions
- Dispute Chargebacks

## Design Goals
- Stable Rust
- Zero dependencies (apart from Rust's standard library)
- Fault tolerance (unrecognised or incorrect transactions are reported and skipped)
- Resistance to malicious input (if the incoming input does not produce a valid transaction, the process stops)

## Usage Instructions

To run mini-payments:
```sh
cargo run -- input_file.txt
```
with the input displayed on **stdout**

To see the docs:
```sh
cargo doc --open
```
which will also describe the transactions spec

To run the tests:
```sh
cargo test
```
Unit tests will also descirbe the components' behaviour