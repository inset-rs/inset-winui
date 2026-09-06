# reveal-winui

WinUI 3 (Fluent 2) on the reveal framework, ported from Microsoft's open source. The spec is the repository at `/Users/mac/code/microsoft-ui-xaml`; values and behaviour are read from it, not remembered or measured from screenshots, so that later controls do not drift from the source. Desktop first: WinUI adapts by window size, not by device, so there is no mobile idiom.

## The spec

- Templates and visual states: `controls/dev/CommonStyles/<Control>_themeresources.xaml` (the Fluent styles; ignore `*_perf2026.xaml`). Older base templates: `dxaml/xcp/dxaml/themes/generic.xaml`.
- Tokens: `controls/dev/CommonStyles/Common_themeresources_any.xaml` (colours per theme, elevation brushes, durations, key splines), `CornerRadius_themeresources.xaml`, `TextBlock_themeresources.xaml`.
- Behaviour: `dxaml/xcp/dxaml/lib/<Control>_Partial.cpp`, `dxaml/xcp/core/core/elements/<Control>.cpp`; WinUI 2-era controls under `controls/dev/<Control>/`.
- Not in the repository: theme-animation timings (`RepositionThemeAnimation` and friends read the OS animation library through `uxtheme`), the accent palette and system colours, Mica's wallpaper processing, the RichEdit text engine, DirectManipulation.

## How the code is shaped

- `theme/generated.rs` holds every colour, brush, size, duration and spline of the XAML theme dictionaries, produced by `tools/gen_resources.py`; `{ThemeResource X}` in a template is the field `x`. It is generated rather than typed so a value cannot be mistyped or invented; a control whose resources are missing is added to the generator's arguments.
- A control file is a transcription of its `ControlTemplate`: the elements in order with their `x:Name`s in comments, each visual state group as an enum, the storyboards as the animations they declare. Its behaviour follows the C++ as closely as reveal allows.
- What the repository leaves to Windows (accent shades, system colours, theme-animation timings, Segoe UI) has a substitute and a `PORTING.md` entry saying so. Text is Selawik, Microsoft's metric-compatible open substitute for Segoe UI, bundled with the gallery.
- Widget construction follows `docs/widget-syntax.md`.

## Porting a control

1. Generate its resources and add a `ThemeResources::<control>()` accessor.
2. Transcribe the template from `<Control>_themeresources.xaml`.
3. Take behaviour from `<Control>_Partial.cpp`.
4. Add a gallery section and a GPU test, and look at the exported screenshot.
5. Write the `PORTING.md` entries.

## Rules

- No stubs, hacks or partial controls: stop and say so.
- Do not commit unless asked.
- Do not add a rule here without asking.

## PORTING.md

Each source folder that diverges from WinUI keeps a `PORTING.md`. An entry is Change / Reason / Affect, for a reader who knows Rust and WinUI's public surface; no visible Affect means identical, so no entry. Deferred items go under `## Deferred`, each with a trigger. The Reason is one of three, because each has its own way back:

- `os` — Windows supplies it and this host does not: a value not in the repository, or a facility of the OS. Name the substitute; the trigger is a measurement or table from a Windows machine, or a host that provides it.
- `framework` — reveal has no equivalent of the XAML mechanism. The trigger is the framework feature.
- `language` — Rust has no counterpart of the XAML or C++ shape. The Affect says what a caller writes differently.
