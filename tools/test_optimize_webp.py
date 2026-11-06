"""Validate the publication gate against real static and animated WebP files."""
import io
import json
from pathlib import Path
import tempfile
import unittest
import hashlib
import optimize_webp as webp


def fixture():
    image = webp.Image.new('RGBA', (128, 128))
    image.putdata([(x % 8 * 31, y % 8 * 31, (x + y) % 8 * 31, 255 if x % 3 else 0)
                   for y in range(128) for x in range(128)])
    data = io.BytesIO()
    image.save(data, 'WEBP', lossless=True, exact=True, method=0, quality=0)
    return data.getvalue()


class WebpTests(unittest.TestCase):
    def test_static_keeps_hidden_rgb_and_pixels(self):
        original = fixture()
        self.assertEqual(webp.signature(original), webp.signature(webp.recompress(original)))

    def test_animation_keeps_frames_timing_and_loop(self):
        first = webp.Image.new('RGBA', (16, 16), (23, 47, 83, 255))
        second = webp.Image.new('RGBA', (16, 16), (87, 43, 29, 128))
        data = io.BytesIO()
        first.save(data, 'WEBP', save_all=True, append_images=[second], lossless=True,
                   exact=True, duration=[37, 83], loop=2)
        self.assertEqual(webp.signature(data.getvalue()), webp.signature(webp.recompress(data.getvalue())))

    def test_gate_requires_write_then_updates_inventory_and_caches(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'assets/ui').mkdir(parents=True)
            (root / 'config').mkdir()
            source = root / 'assets/ui/test.webp'
            original = fixture()
            source.write_bytes(original)
            manifest = root / 'config/asset-manifest.json'
            manifest.write_text(json.dumps({'assets/ui/test.webp': {'sha256': hashlib.sha256(original).hexdigest(), 'bytes': len(original)}}))
            locks = root / 'config/normal-art-lock.json'
            locks.write_text('{"files":{}}')
            with self.assertRaisesRegex(ValueError, 'can save'):
                webp.optimize(root)
            self.assertEqual(source.read_bytes(), original)
            webp.optimize(root, write=True)
            updated = source.read_bytes()
            self.assertLess(len(updated), len(original))
            self.assertEqual(webp.signature(updated), webp.signature(original))
            self.assertEqual(json.loads(manifest.read_text())['assets/ui/test.webp']['sha256'], hashlib.sha256(updated).hexdigest())
            webp.optimize(root)
            # A changed source cannot reuse a previous successful cache entry.
            source.write_bytes(original)
            with self.assertRaisesRegex(ValueError, 'can save'):
                webp.optimize(root)
            locks.write_text(json.dumps({'files': {'assets/ui/test.webp': hashlib.sha256(original).hexdigest()}}))
            webp.optimize(root, write=True)
            self.assertEqual(source.read_bytes(), original)


if __name__ == '__main__':
    unittest.main()
