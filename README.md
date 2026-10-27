# Inset WinUI

> **This repository is deprecated.** `inset-winui` now lives in the Inset main repository at [`crates/inset-winui`](https://github.com/inset-rs/inset/tree/main/crates/inset-winui).

WinUI controls and Fluent styling for [Inset](https://github.com/inset-rs/inset).

The library ports control templates, theme resources and behavior from [Microsoft’s WinUI](https://github.com/microsoft/microsoft-ui-xaml). Inset handles rendering, layout and input.

## Try it

Inset **requires nightly Rust**:

```sh
rustup install nightly
cargo run -p winui_gallery
```

Run the gallery from this repository. Dependencies come from crates.io.

## Use it

```toml
[dependencies]
inset-winui = "0.6.0"
```

Use controls inside an Inset `WidgetsApp`, with `ThemeScope` selecting the light or dark theme. Call `install_icon_font` once during application setup. Controls that show popups need an `Overlay` ancestor.

A `ThemeScope` can also change WinUI's theme resources for its subtree, as XAML's resource dictionaries do: `any` in both themes, then `light` or `dark`. Each resource is a field named after its XAML key.

```rust
ThemeScope::new(Theme::Light, app)
    .any(|r| r.control_corner_radius = [6.0; 4])
    .dark(|r| r.text_fill_color_primary = Color::from_argb(255, 240, 240, 240))
```

`ThemeScope::overriding(child)` changes the enclosing scope's resources for one control, as `<Control.Resources>` does.

The [gallery](https://github.com/inset-rs/inset-winui/tree/main/examples/winui_gallery) demonstrates application setup, font registration and control state.

## Contributing

Porting guidance, tests and resource generation: [AGENTS.md](https://github.com/inset-rs/inset-winui/blob/main/AGENTS.md).

## License

MIT. Microsoft’s source and the bundled icon font retain their [third-party notices](https://github.com/inset-rs/inset-winui/blob/main/crates/inset-winui/THIRD_PARTY.md).
