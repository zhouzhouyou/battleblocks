"""Normal-art protection must work on Windows without weakening the lock."""
import unittest
from check_legacy_normal_unchanged import ROOT, check, mutation_checks


class NormalArtLockTests(unittest.TestCase):
    def test_checkout_line_endings_do_not_change_normal_metadata(self):
        for newline in (b'\n', b'\r\n'):
            def read(name):
                data=(ROOT/name).read_bytes()
                if name.endswith(('.rs','.json')):
                    data=data.replace(b'\r\n',b'\n').replace(b'\n',newline)
                return data
            self.assertEqual(check(read)['files'],14)

    def test_actual_art_timing_order_size_and_routing_changes_are_rejected(self):
        self.assertEqual(set(mutation_checks()),{'duration','order','pixels','registry','routing'})


if __name__=='__main__':
    unittest.main()
