"""Run with python3 -m unittest discover -s tools -p 'test_*.py'."""
import tempfile
import unittest
import contextlib
import io
from pathlib import Path

from gen_resources import control_source, emit_struct, plain_value, read_dictionaries, resolve


class ResourceGenerationTests(unittest.TestCase):
    def test_control_source_prefers_common_styles_then_control_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            local = root / 'controls/dev/SplitView/SplitView_themeresources.xaml'
            common = root / 'controls/dev/CommonStyles/SplitView_themeresources.xaml'
            local.parent.mkdir(parents=True)
            common.parent.mkdir(parents=True)
            local.touch()
            self.assertEqual(control_source(root, 'SplitView'), local)
            common.touch()
            self.assertEqual(control_source(root, 'SplitView'), common)
            with self.assertRaises(FileNotFoundError):
                control_source(root, 'Missing')

    def test_legacy_filter_reads_alias_dependencies_without_unrelated_resources(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'resources.xaml'
            path.write_text('''<ResourceDictionary
                xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
                <ResourceDictionary.ThemeDictionaries>
                  <ResourceDictionary x:Key="Light">
                    <StaticResource x:Key="Overlay" ResourceKey="Brush" />
                    <SolidColorBrush x:Key="Brush" Color="{StaticResource Color}" />
                    <Color x:Key="Color">#99FFFFFF</Color>
                    <Color x:Key="UnrelatedUnsupportedColor">NotAColor</Color>
                  </ResourceDictionary>
                  <ResourceDictionary x:Key="HighContrast">
                    <Color x:Key="Color">NotAColor</Color>
                  </ResourceDictionary>
                </ResourceDictionary.ThemeDictionaries>
                </ResourceDictionary>''')
            themes, _ = read_dictionaries(path, ['Overlay'])
            self.assertEqual(set(themes['Light']), {'Overlay', 'Brush', 'Color'})
            self.assertEqual(resolve({}, themes['Light'], 'Overlay'), ('color', (153, 255, 255, 255)))
            self.assertEqual(resolve({'Color': ('color', (255, 0, 0, 0))}, themes['Light'], 'Overlay'),
                             ('color', (255, 0, 0, 0)))

    def test_animation_pre_duration_keeps_submillisecond_precision(self):
        self.assertEqual(plain_value('String', '00:00:00.19999'), ('duration_ns', 199990000))
        self.assertEqual(plain_value('String', '00:00:00.2'), ('duration_ms', 200))

    def test_unresolved_resource_names_are_reported(self):
        errors = io.StringIO()
        with contextlib.redirect_stderr(errors):
            emit_struct([], 'TestResources', ['MissingBrush'], {}, {}, {}, 'Test.xaml')
        self.assertIn('Test.xaml: unresolved resource MissingBrush (Light, Default)', errors.getvalue())


if __name__ == '__main__':
    unittest.main()
