# WeAct Display FS Driver

Rust driver and test CLI for official WeAct Display FS USB displays.

Currently supported:

- WeAct Studio Display FS 0.96 Inch (`B_80x160`, native 80x160)
- WeAct Studio Display FS V1 (`A_320x480`, native 320x480)

The workspace has three parts:

- `crates/weact-display`: core display protocol, framebuffer, and driver.
- `crates/weact-display-serial`: serial-port transport for real hardware.
- `apps/weact-cli`: small CLI for testing the display.

## Build

```bash
cargo build
```

Run tests:

```bash
cargo test
```

## Find The Display

List all supported WeAct Display FS serial ports:

```bash
cargo run -p weact-cli -- find-port
```

## Fill The Screen

With automatic port lookup:

```bash
cargo run -p weact-cli -- fill --color red
```

With an explicit port:

```bash
cargo run -p weact-cli -- fill --port /dev/ttyACM1 --color red
```

Optional flags:

```bash
--color red|green|blue|black|white
--orientation portrait|landscape|portrait-flipped|landscape-flipped
--brightness 0..100
```

The CLI infers the display model from USB metadata. If several supported displays are connected,
pass the desired serial device explicitly with `--port`.
