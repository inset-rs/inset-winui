# reveal-winui

WinUI 3's controls, Fluent 2 look and motion on the reveal framework, ported from Microsoft's open source (`../microsoft-ui-xaml`). Desktop first; WinUI itself adapts by window size rather than by device, so there is no separate mobile idiom.

Sibling checkouts this workspace expects: `../reveal-rs` (the framework, path dependencies), `../valo` (the renderer, on the branch reveal-rs is pinned to), `../microsoft-ui-xaml` (the spec).

```sh
cargo run -p winui-gallery
cargo test
APPLE_CAPTURE_DIR=$PWD/output cargo test -p winui_gallery   # export what the tests rendered
python3 tools/gen_resources.py ../microsoft-ui-xaml crates/reveal-winui/src/theme/generated.rs Button ToggleSwitch CheckBox TextBlock
```

`AGENTS.md` has the porting rules; `crates/reveal-winui/README.md` the shape of the code.
