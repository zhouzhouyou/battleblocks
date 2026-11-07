"""Check build/test separation without compiling or packaging the game."""
from pathlib import Path
import runpy
import sys
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[1]
FONT_SHA256 = 'a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da'


class BuildTests(unittest.TestCase):
    def commands(self, *arguments):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            script = root / 'tools/build.py'
            script.parent.mkdir()
            script.write_text((ROOT / 'tools/build.py').read_text())
            cache = root / '.build-tools'
            (cache / 'site/fonttools-4.67.0.dist-info').mkdir(parents=True)
            (cache / 'art-site/pillow-13.0.0.dist-info').mkdir(parents=True)
            (cache / 'NotoSansSC.ttf').write_bytes(b'cached font fixture')
            with patch.object(sys, 'argv', [str(script), *arguments]), \
                    patch('hashlib.sha256') as digest, \
                    patch('shutil.which', side_effect=lambda name: name), \
                    patch('subprocess.run') as run:
                digest.return_value.hexdigest.return_value = FONT_SHA256
                try:
                    runpy.run_path(str(script), run_name='__main__')
                except SystemExit as error:
                    self.assertEqual(error.code, 0)
                return [[str(arg) for arg in call.args[0]] for call in run.call_args_list]

    def test_default_build_keeps_policy_checks_without_tests(self):
        commands = self.commands()
        self.assertFalse(any('test' in command or any('/test_' in arg for arg in command) for command in commands))
        self.assertTrue(any(arg.endswith('/tools/check_repository.py') for cmd in commands for arg in cmd))
        self.assertTrue(any(arg.endswith('/tools/optimize_webp.py') for cmd in commands for arg in cmd))
        builds = [cmd for cmd in commands if cmd[:2] == ['cargo', 'build']]
        self.assertEqual(len(builds), 2)
        self.assertTrue(commands[-1][-1].endswith('/tools/package.py'))

    def test_explicit_tests_and_target_selection(self):
        for target in ('--native-only', '--web-only'):
            with self.subTest(target=target):
                commands = self.commands(target, '--test')
                self.assertIn(['cargo', 'test', '--locked'], commands)
                for name in ('test_build.py', 'test_balance.py', 'test_optimize_webp.py'):
                    self.assertTrue(any(arg.endswith('/tools/' + name) for cmd in commands for arg in cmd))
                builds = [cmd for cmd in commands if cmd[:2] == ['cargo', 'build']]
                self.assertEqual(len(builds), 1)
                self.assertEqual('--target' in builds[0], target == '--web-only')
                self.assertEqual(commands[-1][-1], target)

    def test_setup_does_not_build_or_test(self):
        self.assertEqual(self.commands('--setup-only', '--test'), [])


if __name__ == '__main__':
    unittest.main()
