"""Keep the unmodified Microsoft shader bound to its pinned source bytes."""
import hashlib
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ShaderProvenanceTests(unittest.TestCase):
    def test_original_microsoft_shader_is_unchanged(self):
        data = (ROOT / "shader/tests/fixtures/microsoft-hello-triangle.hlsl").read_bytes()
        self.assertEqual(hashlib.sha256(data).hexdigest(),
                         "c300fc6586b8e0410ea3df4c00d080efeeeeac7e0e2485a84864fae118f6597e")


if __name__ == "__main__":
    unittest.main()
