"""Frozen native contract/transport checks; not original-game parity."""
import struct
import sys
import unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_system_windows_directory as observer

class SystemWindowsDirectoryContractTests(unittest.TestCase):
    def test_native_capture_has_all_18_words_and_its_actual_separate_root(self):
        record=observer.frozen();data=bytes.fromhex(record['stdout_hex'])
        words=struct.unpack('<18I',data[:72])
        self.assertEqual(words[0:4],(0x53443332,4,1,0xface1234))
        self.assertEqual(observer.decode(data),record['system_windows_directory'])
        self.assertEqual(record['system_windows_directory'].casefold(),record['windows_root'].casefold())
        self.assertEqual(set(record['inputs']['files']),{'system-windows-directory.exe'})

    def test_changed_guards_errors_truncation_or_wrong_root_cannot_pass(self):
        data=bytes.fromhex(observer.frozen()['stdout_hex'])
        for index in [2,3,6,8,15,17]:
            changed=bytearray(data);struct.pack_into('<I',changed,index*4,0)
            with self.assertRaises(ValueError):observer.decode(changed)
        for n in [0,1,71,75,len(data)-1]:
            with self.assertRaises(ValueError):observer.decode(data[:n])
        with self.assertRaises(ValueError):observer.decode(data,'Z:\\unrelated\\System32')

if __name__=='__main__':unittest.main()
