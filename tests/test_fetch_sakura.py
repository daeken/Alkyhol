import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from tools import fetch_sakura

class SakuraIntakeTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        (self.root/'sakura').mkdir()
        self.payload=b'public sample asset\n'
        self.manifest={'revision':'fixed-pin','directory':'sample/path','assets':{'tree.obj':{'sha256':hashlib.sha256(self.payload).hexdigest(),'bytes':len(self.payload)}}}
        (self.root/'sakura/upstream.json').write_text(json.dumps(self.manifest))
        self.root_patch=patch.object(fetch_sakura,'ROOT',self.root)
        self.root_patch.start();self.addCleanup(self.root_patch.stop)
    def test_verified_download_cache_and_corrupt_cache_repair(self):
        with patch.object(fetch_sakura.urllib.request,'urlopen',return_value=io.BytesIO(self.payload)) as network:
            destination=fetch_sakura.fetch()
            self.assertEqual((destination/'tree.obj').read_bytes(),self.payload)
            self.assertIn('/fixed-pin/sample/path/tree.obj',network.call_args.args[0])
        with patch.object(fetch_sakura.urllib.request,'urlopen',side_effect=AssertionError('cached asset must not fetch')):
            self.assertEqual(fetch_sakura.fetch(),destination)
        (destination/'tree.obj').write_bytes(b'corrupt')
        with patch.object(fetch_sakura.urllib.request,'urlopen',return_value=io.BytesIO(self.payload)):
            fetch_sakura.fetch(destination)
        self.assertEqual((destination/'tree.obj').read_bytes(),self.payload)
    def test_hash_or_size_mismatch_never_replaces_existing_file(self):
        destination=self.root/'custom';destination.mkdir();path=destination/'tree.obj';path.write_bytes(b'existing invalid asset')
        for payload in [self.payload+b'x',b'X'+self.payload[1:],b'']:
            with self.subTest(payload=payload), patch.object(fetch_sakura.urllib.request,'urlopen',return_value=io.BytesIO(payload)):
                with self.assertRaisesRegex(ValueError,'hash mismatch'):
                    fetch_sakura.fetch(destination)
                self.assertEqual(path.read_bytes(),b'existing invalid asset')
                self.assertFalse((destination/'tree.obj.tmp').exists())
