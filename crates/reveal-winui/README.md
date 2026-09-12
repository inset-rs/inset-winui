# reveal-winui

WinUI 3's controls on reveal, ported from `/Users/mac/code/microsoft-ui-xaml`. See the workspace `AGENTS.md` for where each kind of fact lives in that repository and the rules.

## Shape of the code

- `theme/`: `Theme` (light, dark), `AccentPalette` (the OS accent and its shades), `ThemeResources` with `ThemeScope` (XAML's theme dictionaries as an inherited widget; `ThemeResources::of` is `{ThemeResource X}`), `generated.rs` (every colour, brush, size, duration and spline of the XAML theme dictionaries, produced by `tools/gen_resources.py`; never edited by hand) and `typography.rs` (the `*TextBlockStyle` ramp in the bundled Selawik).
- `primitives/`: what templates are made of. `Brush` (solid colours, elevation gradients and in-window acrylic), `ControlBorder` (`Background`, `BorderBrush`, `BorderThickness`, `CornerRadius`, `BackgroundSizing`, `Padding`), `ColorTransition` (`BrushTransition`), `CommonStates` (the `Normal` / `PointerOver` / `Pressed` / `Disabled` group computed from the pointer and keyboard), `FocusVisual` (the system focus rings).
- `controls/`: one file per control, each a transcription of its `ControlTemplate`: the element tree, the visual states as a match over `CommonState`, the storyboards as the animations they declare. `controls/grid/` is XAML's `Grid` panel (`grid.cpp`): `GridLength`, `RowDefinition`, `ColumnDefinition`, `Grid` with spacing and chrome, `GridCell` for the attached `Grid.Row` / `Grid.Column` / spans; `layout.rs` is the measure and arrange algorithm as a pure function, `render_grid.rs` the render object around it.

`SplitView` supports Overlay, Inline, CompactOverlay and CompactInline modes. The owner stores `is_pane_open` and applies the callback's requested changes. Opening an overlay mode requires a native `Overlay` ancestor so taps outside an embedded SplitView can dismiss its pane; the gallery supplies one in its `WidgetsApp` builder. `PaneClosing` can cancel light dismissal; explicit owner changes to `false` still close. Use `f64::NAN` for content-sized `OpenPaneLength`, and a `GlobalKey` to read `SplitViewState::template_settings` when another template needs the resolved lengths.

`TabView` takes stable-id `TabViewItem` snapshots and owner-managed selection, close, reorder and transfer callbacks. `NavigationView` takes stable-id hierarchical items and nullable selection, with adaptive left or top presentation. Both retain item/page state through their native keyed trees. Navigation popups and tab drag feedback need a native `Overlay` ancestor. Install the bundled Fluent icon font alongside the application's text fonts; the gallery shows this setup and working owner callbacks.

## Adding a control

1. Add its `<Control>` to the generator's arguments and regenerate `theme/generated.rs`; the workspace `README.md` carries the current command line, and the generator runs `rustfmt` on its output. Add a `ThemeResources::<control>()` accessor.
2. Read `controls/dev/CommonStyles/<Control>_themeresources.xaml`: transcribe the template's elements in order (name them in comments), a XAML `Grid` as a `Grid` with the same definitions and `GridCell` placements (a child aligned or sized inside its cell gets an `Align` / `SizedBox`), take every brush from the generated resources by state, and every duration and spline from the generated constants.
3. Read `dxaml/xcp/dxaml/lib/<Control>_Partial.cpp` for behaviour (what toggles, when `Click` fires, drag thresholds).
4. Add a gallery section and a GPU test, and an entry in the folder's `PORTING.md` for anything the source leaves to the OS.
