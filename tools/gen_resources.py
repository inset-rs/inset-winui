#!/usr/bin/env python3
"""Generates Rust theme resources from WinUI's XAML theme dictionaries.

usage: gen_resources.py <microsoft-ui-xaml root> <output .rs> <Control ...>

Reads controls/dev/CommonStyles/Common_themeresources_any.xaml (the Fluent base
tokens) and one <Control>_themeresources.xaml per named control, preferring
CommonStyles and then controls/dev/<Control>. Missing shared aliases resolve
through Common_themeresources.xaml and the base generic.xaml dictionary.
Emits one Rust struct per file with
a `for_theme` constructor for the Light and Dark (XAML "Default") dictionaries.
High contrast is not emitted: its values are the OS's system colours.
"""
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

NS = {
    'x': 'http://schemas.microsoft.com/winfx/2006/xaml',
    '': 'http://schemas.microsoft.com/winfx/2006/xaml/presentation',
}
# MenuFlyoutPresenter::UpdateTemplateSettings looks up this key in C++, not XAML.
RUNTIME_RESOURCE_REFERENCES = {'MenuFlyout': ['FlyoutThemeTouchMinWidth']}

X_KEY = '{%s}Key' % NS['x']
P = '{%s}' % NS['']


def local(tag):
    return tag.split('}')[-1]


def parse_color(text):
    text = text.strip()
    named = {'Transparent': (0, 0, 0, 0), 'White': (255, 255, 255, 255), 'Black': (255, 0, 0, 0)}
    if text in named:
        return named[text]
    digits = text.lstrip('#')
    if len(digits) == 6:
        digits = 'FF' + digits
    a, r, g, b = (int(digits[i:i + 2], 16) for i in range(0, 8, 2))
    return (a, r, g, b)


def resource_references(element):
    references = []
    for node in element.iter():
        if local(node.tag) == 'StaticResource' and node.get('ResourceKey'):
            references.append(node.get('ResourceKey'))
        for value in node.attrib.values():
            references.extend(re.findall(r'\{(?:StaticResource|ThemeResource) (\w+)\}', value))
    return references


def control_source(root, control):
    filename = f'{control}_themeresources.xaml'
    candidates = (root / 'controls/dev/CommonStyles' / filename,
                  root / 'controls/dev' / control / filename)
    for path in candidates:
        if path.is_file():
            return path
    nested = list((root / 'controls/dev').rglob(filename))
    if len(nested) == 1:
        return nested[0]
    raise FileNotFoundError(f'No unique resource dictionary for {control}: {candidates}, {nested}')


def read_dictionaries(path, keys=None):
    """Returns {theme: {key: (kind, value)}} for one resource file."""
    root = ET.parse(path).getroot()
    # Legacy generic.xaml contains controls we do not port. Read just the
    # requested resources and their alias dependencies when used as a fallback.
    if keys is not None:
        keys = set(keys)
        while True:
            dependencies = set()
            for element in root.iter():
                if element.get(X_KEY) in keys:
                    dependencies.update(resource_references(element))
            if dependencies <= keys:
                break
            keys.update(dependencies)
    themes = {}
    for theme_dict in root.iter(P + 'ResourceDictionary'):
        key = theme_dict.get(X_KEY)
        if key not in ('Default', 'Light'):
            continue
        entries = themes.setdefault(key, {})
        for element in theme_dict:
            name = element.get(X_KEY)
            if name is None or (keys is not None and name not in keys):
                continue
            kind = local(element.tag)
            if kind == 'StaticResource':
                entries[name] = ('alias', element.get('ResourceKey'))
            elif kind == 'SolidColorBrush':
                color = element.get('Color', '')
                match = re.match(r'\{(?:StaticResource|ThemeResource) (\w+)\}', color)
                entries[name] = ('alias', match.group(1)) if match else ('color', parse_color(color))
            elif kind == 'Color':
                entries[name] = ('color', parse_color(element.text))
            elif kind == 'AcrylicBrush':
                entries[name] = ('acrylic', {
                    'tint_color': element.get('TintColor', 'White'),
                    'tint_opacity': float(element.get('TintOpacity', '0.5')),
                    'fallback_color': element.get('FallbackColor', 'Transparent'),
                    'tint_luminosity_opacity': element.get('TintLuminosityOpacity'),
                })
            elif kind == 'LinearGradientBrush':
                stops = []
                for stop in element.iter(P + 'GradientStop'):
                    color = stop.get('Color')
                    match = re.fullmatch(r'\{(?:StaticResource|ThemeResource) (\w+)\}', color)
                    stops.append((float(stop.get('Offset')), match.group(1) if match else ('color', parse_color(color))))
                entries[name] = ('gradient', stops)
            else:
                # Sizes inside a theme dictionary that every theme agrees on
                # are emitted once, as constants.
                plain = plain_value(kind, (element.text or '').strip())
                if plain is not None:
                    entries[name] = plain
    # Values outside the theme dictionaries apply to every theme.
    shared = {}
    for element in root:
        name = element.get(X_KEY)
        if name is None or (keys is not None and name not in keys):
            continue
        plain = plain_value(local(element.tag), (element.text or '').strip())
        if plain is not None:
            shared[name] = plain
    # A size both themes define identically is a constant too.
    light, dark = themes.get('Light', {}), themes.get('Default', {})
    for name, value in light.items():
        if value[0] in ('f64', 'thickness', 'corner', 'duration_ms', 'duration_ns', 'spline', 'enum') and dark.get(name) == value:
            shared.setdefault(name, value)
    return themes, shared


def plain_value(kind, text):
    if kind == 'Double':
        return ('f64', float(text))
    if kind == 'Thickness':
        parts = [float(v) for v in text.split(',')]
        if len(parts) == 1:
            parts = parts * 4
        elif len(parts) == 2:
            parts = [parts[0], parts[1], parts[0], parts[1]]
        return ('thickness', parts)
    if kind == 'CornerRadius':
        parts = [float(v) for v in text.split(',')]
        return ('corner', parts if len(parts) == 4 else parts * 4)
    if kind == 'String' and re.match(r'\d\d:\d\d:\d\d', text):
        h, m, s = text.split(':')
        nanoseconds = round((float(s) + float(m) * 60 + float(h) * 3600) * 1_000_000_000)
        return ('duration_ms', nanoseconds // 1_000_000) if nanoseconds % 1_000_000 == 0 else ('duration_ns', nanoseconds)
    if kind == 'String' and re.match(r'^[\d., -]+$', text):
        return ('spline', [float(v) for v in text.split(',')])
    if kind in ('BackgroundSizing', 'FontWeight'):
        # Enum-valued resources: the Rust enum shares the XAML names.
        return ('enum', (kind, text.strip()))
    return None


ACCENT = ('SystemAccentColor', 'SystemAccentColorLight1', 'SystemAccentColorLight2', 'SystemAccentColorLight3',
          'SystemAccentColorDark1', 'SystemAccentColorDark2', 'SystemAccentColorDark3')


def resolve(entries, base, name, seen=()):
    entry = entries.get(name) or base.get(name)
    if entry is None:
        # The accent palette comes from the OS at run time; it is an input of
        # `for_theme`, not a literal.
        return ('accent', name) if name in ACCENT else None
    kind, value = entry
    if kind == 'alias':
        if value in seen:
            return None
        return resolve(entries, base, value, seen + (value,))
    if kind == 'gradient':
        return ('gradient', [(offset, key if isinstance(key, tuple) else resolve(entries, base, key)) for offset, key in value])
    if kind == 'acrylic':
        value = dict(value)
        for field in ('tint_color', 'fallback_color'):
            match = re.fullmatch(r'\{(?:StaticResource|ThemeResource) (\w+)\}', value[field])
            value[field] = resolve(entries, base, match.group(1), seen) if match else ('color', parse_color(value[field]))
            if value[field] is None:
                return None
        return ('acrylic', value)
    return entry


def snake(name):
    out = re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', name).lower()
    return out


def color_literal(color):
    a, r, g, b = color
    return f'Color::from_argb({a}, {r}, {g}, {b})'


def accent_field(name):
    return {'SystemAccentColor': 'accent.base', 'SystemAccentColorLight1': 'accent.light1',
            'SystemAccentColorLight2': 'accent.light2', 'SystemAccentColorLight3': 'accent.light3',
            'SystemAccentColorDark1': 'accent.dark1', 'SystemAccentColorDark2': 'accent.dark2',
            'SystemAccentColorDark3': 'accent.dark3'}[name]


def value_literal(entry):
    kind, value = entry
    return accent_field(value) if kind == 'accent' else color_literal(value)


def acrylic_literal(value):
    luminosity = value['tint_luminosity_opacity']
    luminosity = f'Some({float(luminosity)})' if luminosity is not None else 'None'
    return ('AcrylicBrushResources { '
            f'tint_color: {value_literal(value["tint_color"])}, '
            f'tint_opacity: {value["tint_opacity"]}, '
            f'tint_luminosity_opacity: {luminosity}, '
            f'fallback_color: {value_literal(value["fallback_color"])} }}')


def emit_struct(out, struct_name, keys, themes, base_themes, shared, source):
    fields = []
    values = {'Light': [], 'Default': []}
    for key in keys:
        light = resolve(themes.get('Light', {}), base_themes.get('Light', {}), key)
        dark = resolve(themes.get('Default', {}), base_themes.get('Default', {}), key)
        if light is None or dark is None:
            missing = ', '.join(theme for theme, value in (('Light', light), ('Default', dark)) if value is None)
            print(f'warning: {source}: unresolved resource {key} ({missing}); not emitted', file=sys.stderr)
            continue
        kinds = {light[0], dark[0]}
        if kinds <= {'color', 'accent'}:
            fields.append((snake(key), 'Color'))
            values['Light'].append(value_literal(light))
            values['Default'].append(value_literal(dark))
        elif kinds == {'gradient'}:
            stops = lambda g: '[' + ', '.join(f'({o}, {value_literal(c)})' for o, c in g if c) + ']'
            fields.append((snake(key), '[(f64, Color); 2]'))
            values['Light'].append(stops(light[1]))
            values['Default'].append(stops(dark[1]))
        elif kinds == {'acrylic'}:
            fields.append((snake(key), 'AcrylicBrushResources'))
            values['Light'].append(acrylic_literal(light[1]))
            values['Default'].append(acrylic_literal(dark[1]))
        elif light != dark:
            # A size, duration or spline each theme states differently has no
            # constant to be emitted as, and the struct holds brushes only.
            print(f'warning: {source}: {key} differs by theme ({light[1]} light, {dark[1]} dark); not emitted', file=sys.stderr)
    out.append(f'/// Theme-dependent resources of `{source}`, resolved to literals; `Default` in XAML is the dark theme.')
    out.append('#[derive(Clone, Debug, PartialEq)]')
    out.append(f'pub struct {struct_name} {{')
    for field, ty in fields:
        out.append(f'    /// The resolved `{next(key for key in keys if snake(key) == field)}` resource.')
        out.append(f'    pub {field}: {ty},')
    out.append('}')
    out.append('')
    out.append(f'impl {struct_name} {{')
    out.append('    /// Resolves the source dictionary for the requested theme and accent palette.')
    out.append('    pub fn for_theme(theme: Theme, accent: &AccentPalette) -> Self {')
    out.append('        match theme {')
    for theme, label in (('Light', 'Theme::Light'), ('Default', 'Theme::Dark')):
        out.append(f'            {label} => Self {{')
        for (field, _), value in zip(fields, values[theme]):
            out.append(f'                {field}: {value},')
        out.append('            },')
    out.append('        }')
    out.append('    }')
    out.append('}')
    for key, (kind, value) in shared.items():
        name = snake(key).upper()
        if kind == 'f64':
            out.append(f'pub const {name}: f64 = {value};')
        elif kind == 'thickness':
            out.append(f'/// Left, top, right, bottom.\npub const {name}: [f64; 4] = {value};')
        elif kind == 'corner':
            out.append(f'/// Top-left, top-right, bottom-right, bottom-left.\npub const {name}: [f64; 4] = {value};')
        elif kind == 'duration_ms':
            out.append(f'pub const {name}: Duration = Duration::from_millis({value});')
        elif kind == 'duration_ns':
            out.append(f'pub const {name}: Duration = Duration::from_nanos({value});')
        elif kind == 'spline':
            out.append(f'/// Cubic Bézier control points (x1, y1, x2, y2).\npub const {name}: [f64; 4] = {value};')
        elif kind == 'enum':
            enum, variant = value
            # `FontWeight` in reveal is Flutter's: `FontWeight::NORMAL`, `BOLD`, `W600`...
            if enum == 'FontWeight':
                variant = {'Normal': 'NORMAL', 'Bold': 'BOLD', 'SemiBold': 'W600', 'Light': 'W300', 'SemiLight': 'W350', 'Medium': 'W500', 'Black': 'W900', 'Thin': 'W100', 'ExtraLight': 'W200', 'ExtraBold': 'W800'}[variant]
            out.append(f'pub const {name}: {enum} = {enum}::{variant};')


def main():
    root, output, controls = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3:]
    styles = root / 'controls/dev/CommonStyles'
    base_themes, base_shared = read_dictionaries(styles / 'Common_themeresources_any.xaml')
    common_themes, common_shared = read_dictionaries(styles / 'Common_themeresources.xaml')
    base_shared = {**base_shared, **common_shared}
    acrylic_themes, _ = read_dictionaries(root / 'controls/dev/Materials/Acrylic/AcrylicBrush_themeresources.xaml')
    out = ['//! Generated by tools/gen_resources.py from microsoft-ui-xaml; do not edit by hand.',
           '//! Colours are ARGB literals resolved through every StaticResource alias in the',
           '//! XAML theme dictionaries; the XAML "Default" dictionary is the dark theme.',
           '#![allow(clippy::excessive_precision, unused_variables)]',
           'use super::{AccentPalette, AcrylicBrushResources, Theme};', 'use crate::BackgroundSizing;', 'use reveal_embedder::{Color, FontWeight};', 'use std::time::Duration;', '']
    # Brush keys alias a colour rather than stating one, so the alias kind belongs
    # in the shared struct as much as a literal colour does.
    common_keys = [k for k, v in base_themes['Light'].items() if v[0] in ('color', 'gradient', 'alias')]
    emit_struct(out, 'CommonResources', common_keys, base_themes, {}, base_shared, 'Common_themeresources_any.xaml')
    sources = [(control, control_source(root, control)) for control in controls]
    # Control dictionaries share named resources in WinUI's merged theme dictionary.
    # A sibling's Fluent definition must take precedence over generic.xaml fallback.
    control_themes = {'Light': {}, 'Default': {}}
    for _, source in sources:
        themes, _ = read_dictionaries(source)
        for theme in control_themes:
            control_themes[theme].update(themes.get(theme, {}))
    declared = set(base_shared)
    for _, source in sources:
        declared.update(read_dictionaries(source)[1])
    emitted = set(base_shared)
    for control, source in sources:
        themes, shared = read_dictionaries(source)
        templates = [ET.parse(source).getroot()]
        companion = source.with_name(f'{control}.xaml')
        if companion.is_file():
            templates.append(ET.parse(companion).getroot())
        references = [key for template in templates for key in resource_references(template)]
        references.extend(RUNTIME_RESOURCE_REFERENCES.get(control, []))
        legacy_themes, legacy_shared = read_dictionaries(root / 'dxaml/xcp/dxaml/themes/generic.xaml', references)
        shared = {**{key: value for key, value in legacy_shared.items() if key not in declared}, **shared}
        shared = {key: value for key, value in shared.items() if key not in emitted}
        emitted.update(shared)
        fallback = {theme: {**legacy_themes.get(theme, {}), **common_themes.get(theme, {}), **acrylic_themes.get(theme, {}), **base_themes.get(theme, {}), **control_themes.get(theme, {})}
                    for theme in ('Light', 'Default')}
        keys = list(themes.get('Light', {}).keys())
        # Expose system brushes used directly by a template, as well as its
        # named resources. Fluent common tokens retain their existing accessor.
        template_references = references
        for key in template_references:
            if key not in keys and key not in base_themes.get('Light', {}) and (key in legacy_themes.get('Light', {}) or key in control_themes.get('Light', {})):
                keys.append(key)
        struct_name = 'AcrylicThemeResources' if control == 'AcrylicBrush' else f'{control}Resources'
        emit_struct(out, struct_name, keys, themes, fallback, shared, source.name)
    output.write_text('\n'.join(out) + '\n')
    subprocess.run(['rustfmt', '--edition', '2024', str(output)], check=True)
    print(f'wrote {output}')


if __name__ == '__main__':
    main()
