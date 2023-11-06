# v8_valueserializer

This module implements the V8 ValueSerializer and ValueDeserializer API in Rust.
It can serialize and deserialize any value that can be represented in V8's
format.

Additionally this library can generate an eval'able JavaScript representation of
the serialized value that can be used for user display, manipulation, or
debugging.

In the future this library will also provide a way to serialize and deserialize
values to and from JavaScript objects or a structured intermediate in JavaScript
via WebAssembly.

## Development

To build:

```
$ cargo build
```
