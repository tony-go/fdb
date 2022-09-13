# fdb

Fdb is an _append log only_ key value store written in Rust.

Written in the context of "Rust In Action" book.

## API

```bash
Usage:
  │ fdb <FILE> get <KEY>
  │ fbd <FILE> delete <KEY>
  │ fbd <FILE> insert <KEY> <VALUE>
  │ fbd <FILE> update <KEY> <VALUE>
```

If you want to use from source you can use "cargo run" instead of "fdb, for example:

```bash
# cargo run <FILE> get foo

$ cargo run my-db get foo
```

## Install

_TODO_

## Contributing

### Tests

```
$ cargo test -- --test-threads=1
```

> Note: it's important to use the option "test-threads", otherwire tests will fail.
