import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from tools import fetch_miniengine
class MiniEngineIntakeTests(unittest.TestCase):
    def test_nested_asset_hash_verification_cache_and_corruption_rejection(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);(root/'miniengine').mkdir();payload=b'public h3d fixture'
            manifest={'revision':'pinned','directory':'sample','assets':{'Sponza/sponza.h3d':{'bytes':len(payload),'sha256':hashlib.sha256(payload).hexdigest()}}}
            (root/'miniengine/upstream.json').write_text(json.dumps(manifest))
            with patch.object(fetch_miniengine,'ROOT',root):
                with patch.object(fetch_miniengine.urllib.request,'urlopen',return_value=io.BytesIO(payload)) as network:
                    destination=fetch_miniengine.fetch();path=destination/'Sponza/sponza.h3d';self.assertEqual(path.read_bytes(),payload);self.assertIn('/pinned/sample/Sponza/sponza.h3d',network.call_args.args[0])
                with patch.object(fetch_miniengine.urllib.request,'urlopen',side_effect=AssertionError('cache must not fetch')):fetch_miniengine.fetch()
                path.write_bytes(b'old corrupt file')
                with patch.object(fetch_miniengine.urllib.request,'urlopen',return_value=io.BytesIO(payload+b'x')):
                    with self.assertRaisesRegex(ValueError,'hash mismatch'):fetch_miniengine.fetch()
                self.assertEqual(path.read_bytes(),b'old corrupt file')
                with patch.object(fetch_miniengine.urllib.request,'urlopen',return_value=io.BytesIO(payload)):fetch_miniengine.fetch()
                self.assertEqual(path.read_bytes(),payload)
