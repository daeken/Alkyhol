"""Registry witness transport integrity, independent of game/registry equivalence."""
import struct,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_registry_open as registry
class RegistryProtocol(unittest.TestCase):
    def test_every_status_output_guard_and_error_word_is_required(self):
        words=registry.expected_words();data=struct.pack('<26I',*words)
        self.assertEqual(registry.decode(data),words)
        for i in range(26):
            changed=bytearray(data);changed[i*4]^=1
            with self.subTest(word=i),self.assertRaises(ValueError):registry.decode(changed)
    def test_truncated_or_extra_transport_cannot_pass(self):
        data=struct.pack('<26I',*registry.expected_words())
        for changed in [b'',data[:-1],data+b'\0']:
            with self.assertRaises(ValueError):registry.decode(changed)
if __name__=='__main__':unittest.main()
