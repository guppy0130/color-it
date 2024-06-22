# color-it

humans were never meant to parse json with their eyeballs. have some color.

```sh
cat test.jsonl | RUST_LOG=trace cargo run
```

## options

you can pass some options to control:

* what json-log-keys are used
* how time is parsed

e.g., if your payload looks like

```json
{"time": "2024/02/12 12:22 -06:00", "msg": "hello world", "severity": "info"}
```

then you can run

```sh
RUST_LOG=trace color-it --level severity --message msg --timestamp time --strptime '%Y/%m/%d %H:%M %:z'
```

## more notes

* if `timestamp` is an int, it'll be wrapped into a string, so you still have to pass `strptime` (see #1)
* args to `strptime`: <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
* how `level` is `Deserialize`'d: <https://github.com/rust-lang/log/blob/7cb6a01dff9157f3f3dca36aa0152f144023ff60/src/serde.rs#L31>
  * pass an int ([supposedly](https://github.com/rust-lang/log/blob/7cb6a01dff9157f3f3dca36aa0152f144023ff60/src/serde.rs#L45), haven't figured this out yet)
  * pass a [case-insensitive string](https://github.com/rust-lang/log/blob/7cb6a01dff9157f3f3dca36aa0152f144023ff60/src/serde.rs#L61)
