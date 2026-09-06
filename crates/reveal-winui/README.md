# reveal-winui

WinUI 3's controls on reveal, ported from `/Users/mac/code/microsoft-ui-xaml`. See the workspace `AGENTS.md` for where each kind of fact lives in that repository and the rules.

## Shape of the code

- `theme/`: `Theme` (light, dark), `AccentPalette` (the OS accent and its shades), `ThemeResources` with `ThemeScope` (XAML's theme dictionaries as an inherited widget; `ThemeResources::of` is `{ThemeResource X}`), `generated.rs` (every colour, brush, size, duration and spline of the XAML theme dictionaries, produced by `tools/gen_resources.py`; never edited by hand) and `typography.rs` (the `*TextBlockStyle` ramp in the bundled Selawik).
- `primitives/`: what templates are made of. `Brush` (solid and the two elevation gradients), `ControlBorder` (`Background`, `BorderBrush`, `BorderThickness`, `CornerRadius`, `BackgroundSizing`, `Padding`), `ColorTransition` (`BrushTransition`), `CommonStates` (the `Normal` / `PointerOver` / `Pressed` / `Disabled` group computed from the pointer and keyboard), `FocusVisual` (the system focus rings).
- `controls/`: one file per control, each a transcription of its `ControlTemplate`: the element tree, the visual states as a match over `CommonState`, the storyboards as the animations they declare.

## Adding a control

1. Add its `<Control>` to the generator's arguments and regenerate `theme/generated.rs`; add a `ThemeResources::<control>()` accessor.
2. Read `controls/dev/CommonStyles/<Control>_themeresources.xaml`: transcribe the template's elements in order (name them in comments), map `Grid` rows and columns to reveal layout by hand, take every brush from the generated resources by state, and every duration and spline from the generated constants.
3. Read `dxaml/xcp/dxaml/lib/<Control>_Partial.cpp` for behaviour (what toggles, when `Click` fires, drag thresholds).
4. Add a gallery section and a GPU test, and a `PORTING.md` entry for anything the source leaves to the OS.
