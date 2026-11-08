"""Integrity of the open XML parser used by the actual DAO startup bridge."""
import hashlib
import json
from pathlib import Path
import unittest


class XmlVendorTests(unittest.TestCase):
    def test_pinned_crate_source_and_licenses_remain_exact(self):
        root = Path(__file__).resolve().parents[1]/'pe/vendor/roxmltree-0.20.0'
        value = json.loads((root/'UPSTREAM.json').read_text())
        self.assertEqual((value['crate'], value['version']), ('roxmltree', '0.20.0'))
        self.assertEqual(value['archive_sha256'], '6c20b6793b5c2fa6553b250154b78d6d0db37e72700ae35fad9387a46f487c97')
        for name, expected in value['files'].items():
            with self.subTest(name=name):
                self.assertEqual(hashlib.sha256((root/name).read_bytes()).hexdigest(), expected)
        license = value['additional_license']
        self.assertEqual(hashlib.sha256((root/license['file']).read_bytes()).hexdigest(), license['sha256'])
        self.assertTrue({'LICENSE-MIT', 'LICENSE-APACHE', 'src/lib.rs', 'src/parse.rs', 'src/tokenizer.rs'} <= value['files'].keys())


if __name__ == '__main__':
    unittest.main()
