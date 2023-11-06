# v8_valueserializer

This module implements the V8 ValueSerializer and ValueDeserializer API in Rust.
It can serialize and deserialize any value that can be represented in V8's
format.

Additionally this library can generate an eval'able JavaScript representation of
the serialized value that can be used for user display, manipulation, or
debugging.

## Development

To build:

```
$ cargo build
```