"""Synthetic geometry generator/collector contracts; native execution is separate."""
from pathlib import Path
import struct
import subprocess
import sys
import unittest
from unittest import mock
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from build_pe32_geometry import variants,CASES
from run_pe32_geometry import capture


def sample():
    b=bytearray(1024);b[:2]=b'MZ';struct.pack_into('<I',b,0x3c,0x80);b[0x80:0x84]=b'PE\0\0'
    for at,value in [(0x84,0x14c),(0x86,2),(0x94,224),(0x98,0x10b)]:struct.pack_into('<H',b,at,value)
    struct.pack_into('<I',b,0x98+32,4096);struct.pack_into('<I',b,0x98+56,0x3000)
    for i,(virtual,rva,raw) in enumerate([(0x90,0x1000,512),(0x51,0x2000,512)]):struct.pack_into('<III',b,0x98+224+i*40+8,virtual,rva,raw)
    return bytes(b)


class Pe32GeometryTests(unittest.TestCase):
    def test_only_size_field_changes_and_all_cases_keep_distinct_original_headers(self):
        raw=sample();offset,items=variants(raw)
        self.assertEqual([row['case'] for row,_ in items],CASES)
        self.assertEqual([row['declared_size'] for row,_ in items],[0x3000,0x2fff,0x2051,0x2200,0x2001,0x2000,0x1fff,0])
        self.assertEqual(items[0][1],raw)
        for row,data in items:
            self.assertEqual(data[:offset],raw[:offset]);self.assertEqual(data[offset+4:],raw[offset+4:])
            self.assertEqual(struct.unpack_from('<I',data,offset)[0],row['declared_size'])

    def test_wrong_architecture_truncation_or_nondistinct_geometry_is_not_accepted(self):
        for raw in [b'',sample()[:160],b'XX'+sample()[2:]]:
            with self.assertRaises(ValueError):variants(raw)
        for at,fmt,value in [(0x84,'H',0x8664),(0x98,'H',0x20b),(0x98+32,'I',8192),(0x86,'H',0)]:
            raw=bytearray(sample());struct.pack_into('<'+fmt,raw,at,value)
            with self.assertRaises(ValueError):variants(raw)

    def test_process_exit_crash_and_raw_stdout_stderr_are_preserved_without_approval(self):
        for status in [0,17,0xc0000005]:
            run=subprocess.CompletedProcess([],status,b'\x00\xff',b'\x01\x80')
            with mock.patch('run_pe32_geometry.subprocess.run',return_value=run):
                self.assertEqual(capture(['owned.exe'],'unused'),{'kind':'process','status':status,'stdout_hex':'00ff','stderr_hex':'0180'})

    def test_launch_errors_remain_distinct_from_executed_programs_and_timeouts_fail(self):
        error=OSError(8,'owned invalid image');error.winerror=193
        with mock.patch('run_pe32_geometry.subprocess.run',side_effect=error):
            self.assertEqual(capture(['owned.exe'],'unused'),{'kind':'launch_error','winerror':193,'errno':8})
        for error in [OSError('not a measured Windows error'),subprocess.TimeoutExpired('owned.exe',30)]:
            with mock.patch('run_pe32_geometry.subprocess.run',side_effect=error):
                with self.assertRaises(type(error)):capture(['owned.exe'],'unused')


if __name__=='__main__':unittest.main()
