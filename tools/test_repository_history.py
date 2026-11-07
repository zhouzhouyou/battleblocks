"""Exercise history policy in isolated Git repositories, including rejected history."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import check_repository as policy


class RepositoryHistoryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='battleblocks-policy-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git('init', '-q')
        self.git('config', 'user.name', 'Policy Test')
        self.git('config', 'user.email', 'policy@example.invalid')
        self.git('config', 'core.autocrlf', 'false')
        self.git('config', 'commit.gpgsign', 'false')
        self.git('config', 'core.hooksPath', str(self.root / 'no-hooks'))
        self.manifest = {}
        self.inputs = {'config/asset-manifest.json': 'approved assets',
                       'config/repository-inputs.json': 'declared inputs'}

    def git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.root, stderr=subprocess.STDOUT)

    def write(self, name, data):
        target = self.root / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)

    def asset(self, name='assets/roles/test/runtime/atlas.webp', data=b'formal artwork'):
        self.write(name, data)
        self.manifest[name] = {'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}
        return name

    def commit(self, inventories=True):
        if inventories:
            self.write('config/asset-manifest.json', json.dumps(self.manifest).encode())
            self.write('config/repository-inputs.json', json.dumps(self.inputs).encode())
        self.git('add', '.')
        self.git('commit', '-qm', 'test snapshot')

    def check(self):
        with patch.object(policy, 'ROOT', self.root), contextlib.redirect_stdout(io.StringIO()):
            policy.check_history()

    def test_formal_resource_can_be_replaced_then_removed(self):
        name = self.asset()
        self.commit()
        self.asset(name, b'new formal artwork')
        self.commit()
        (self.root / name).unlink()
        del self.manifest[name]
        self.asset('assets/roles/test/runtime/new-atlas.webp')
        self.commit()
        self.check()

    def test_unapproved_old_revision_is_not_retroactively_approved(self):
        self.write('assets/roles/test/runtime/atlas.webp', b'unapproved')
        self.commit()
        self.asset()
        self.commit()
        with self.assertRaisesRegex(SystemExit, 'Unapproved binary revision'):
            self.check()

    def test_corrected_inventory_approves_identical_historical_file(self):
        name = self.asset()
        correct = self.manifest[name].copy()
        self.manifest[name]['sha256'] = '0' * 64
        self.commit()
        self.manifest[name] = correct
        self.commit()
        self.check()

    def test_inventory_without_matching_actual_file_cannot_approve_history(self):
        name = self.asset(data=b'unapproved')
        old = self.manifest[name].copy()
        del self.manifest[name]
        self.commit()
        self.write(name, b'different file')
        self.manifest[name] = old
        self.commit()
        with self.assertRaisesRegex(SystemExit, 'Unapproved binary revision'):
            self.check()

    def test_candidate_is_rejected_even_if_declared_and_later_removed(self):
        name = self.asset('assets/roles/test/runtime/candidate.webp')
        self.commit()
        (self.root / name).unlink()
        del self.manifest[name]
        self.commit()
        with self.assertRaisesRegex(SystemExit, 'Review/reference media'):
            self.check()

    def assert_forbidden(self, name):
        self.write(name, b'{}')
        self.inputs[name] = 'claimed input'
        self.commit()
        (self.root / name).unlink()
        del self.inputs[name]
        self.commit()
        with self.assertRaisesRegex(SystemExit, 'Forbidden generated/one-off history'):
            self.check()

    def test_log_is_rejected_even_if_declared_and_later_removed(self):
        self.assert_forbidden('logs/battle.jsonl')

    def test_report_is_rejected_even_if_declared_and_later_removed(self):
        self.assert_forbidden('docs/run-report.md')

    def test_local_output_is_rejected_even_if_declared_and_later_removed(self):
        self.assert_forbidden('.local/check.json')

    def test_size_must_match_historical_inventory(self):
        name = self.asset()
        self.manifest[name]['bytes'] += 1
        self.commit()
        with self.assertRaisesRegex(SystemExit, 'Unapproved binary revision'):
            self.check()

    def test_hash_must_match_historical_inventory(self):
        name = self.asset()
        self.manifest[name]['sha256'] = '0' * 64
        self.commit()
        with self.assertRaisesRegex(SystemExit, 'Unapproved binary revision'):
            self.check()

    def test_pre_inventory_commits_use_first_inventory(self):
        self.asset()
        self.commit(inventories=False)
        self.commit()
        self.check()

    def test_pre_inventory_unapproved_revision_is_rejected(self):
        self.asset(data=b'never approved')
        self.commit(inventories=False)
        self.asset()
        self.commit()
        with self.assertRaisesRegex(SystemExit, 'Unapproved binary revision'):
            self.check()

    def test_first_inventory_entry_requires_an_actual_matching_file(self):
        name = self.asset()
        self.commit(inventories=False)
        (self.root / name).unlink()
        self.commit()
        with self.assertRaisesRegex(SystemExit, 'Unapproved binary revision'):
            self.check()

    def test_deleted_inventory_does_not_fall_back(self):
        self.asset()
        self.commit()
        (self.root / 'config/asset-manifest.json').unlink()
        self.commit(inventories=False)
        with self.assertRaisesRegex(SystemExit, 'Missing historical inventory'):
            self.check()

    def test_removed_formal_structured_input_uses_its_own_declaration(self):
        name = 'assets-source/roles/test/animation/registration.json'
        self.inputs[name] = 'frame registration'
        self.write(name, b'{}')
        self.commit()
        (self.root / name).unlink()
        del self.inputs[name]
        self.commit()
        self.check()


if __name__ == '__main__':
    unittest.main()
