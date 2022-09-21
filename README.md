# Toy Payments Engine

A simple payments engine that handles deposits, withdrawals, disputes, resolves, and chargebacks. 
The Rust crate provides both a command line interface and a library.

## Usage

Run the CLI via `cargo run` as follows:

```sh
cargo run -- transactions.csv > accounts.csv
```

For the library interface and the module structure, please consult the crate's documentation via `cargo doc --open`.

## Remarks

* Client **accounts** are **created implicitly only for deposits** because all other transactions presuppose at least one deposit and would fail immediately.
* Only deposit transactions can be disputed. This follows from the specification: "the clients available funds should decrease by the amount disputed"
* Disputes may fail if the client's available funds are less than the disputed amount. This is a **loophole that should be fixed**.
* The payments engine is **not thread safe**. Running it in a single thread is the easiest way to ensure consistency (the current transaction sees the effects of all past transactions).
  * If we loosen the consistency requirements such that transactions only need to be applied in order for each client individually, we could partition the data into buckets according to client IDs (or ranges of client IDs) and lock each bucket individually. This would require a redesign.
  