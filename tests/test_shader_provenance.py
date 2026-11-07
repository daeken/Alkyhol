"""Keep the unmodified Microsoft shader bound to its pinned source bytes."""
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ShaderProvenanceTests(unittest.TestCase):
    def test_original_microsoft_shader_is_unchanged(self):
        data = (ROOT / "shader/tests/fixtures/microsoft-hello-triangle.hlsl").read_bytes()
        self.assertEqual(hashlib.sha256(data).hexdigest(),
                         "c300fc6586b8e0410ea3df4c00d080efeeeeac7e0e2485a84864fae118f6597e")

    def test_original_sample_shader_closures_are_unchanged(self):
        for path in (ROOT / "shader/samples").glob("*/upstream.json"):
            manifest=json.loads(path.read_text())
            self.assertEqual(manifest["revision"],"68a2b1e97f71642924e74c7757ca21756ad221b5")
            for name,record in manifest["files"].items():
                data=(path.parent/"original"/name).read_bytes()
                self.assertEqual(len(data),record["bytes"],name)
                self.assertEqual(hashlib.sha256(data).hexdigest(),record["sha256"],name)


if __name__ == "__main__":
    unittest.main()
