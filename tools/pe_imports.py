#!/usr/bin/env python3
"""Dependency-free PE32/PE32+ static and delay import inventory. Never executes input."""
import argparse
import hashlib
import json
from pathlib import Path
import struct


class PEError(ValueError):
    pass


class PE:
    def __init__(self, data):
        self.data = data
        if self.take(0, 2) != b'MZ':
            raise PEError('missing DOS signature')
        pe = self.unpack('<I', 0x3c)[0]
        if self.take(pe, 4) != b'PE\0\0':
            raise PEError('missing PE signature')
        self.machine, count, _, _, _, optional_size, _ = self.unpack('<HHIIIHH', pe + 4)
        optional = pe + 24
        magic = self.unpack('<H', optional)[0]
        if magic not in (0x10b, 0x20b):
            raise PEError('unsupported optional header')
        self.bits = 64 if magic == 0x20b else 32
        self.width = self.bits // 8
        minimum = 112 if self.bits == 64 else 96
        if optional_size < minimum:
            raise PEError('truncated optional header')
        self.image_base = self.unpack('<Q' if self.bits == 64 else '<I', optional + (24 if self.bits == 64 else 28))[0]
        self.headers = self.unpack('<I', optional + 60)[0]
        directory_count = self.unpack('<I', optional + minimum - 4)[0]
        if directory_count > (optional_size - minimum) // 8:
            raise PEError('directories exceed optional header')
        self.directories = [self.unpack('<II', optional + minimum + i * 8) for i in range(directory_count)]
        self.sections = []
        for i in range(count):
            off = optional + optional_size + i * 40
            self.take(off, 40)
            virtual_size, rva, raw_size, raw = self.unpack('<IIII', off + 8)
            self.sections.append((rva, virtual_size, raw, raw_size))

    def take(self, off, size):
        if off < 0 or size < 0 or off + size > len(self.data):
            raise PEError(f'file range out of bounds: {off:#x}+{size:#x}')
        return self.data[off:off + size]

    def unpack(self, fmt, off):
        return struct.unpack(fmt, self.take(off, struct.calcsize(fmt)))

    def offset(self, rva, size=1):
        if 0 <= rva < self.headers and rva + size <= self.headers:
            self.take(rva, size)
            return rva
        for start, virtual, raw, raw_size in self.sections:
            if start <= rva < start + max(virtual, raw_size):
                delta = rva - start
                if delta + size > raw_size:
                    raise PEError('RVA refers to unbacked section bytes')
                self.take(raw + delta, size)
                return raw + delta
        raise PEError(f'unmapped RVA {rva:#x}')

    def string(self, rva):
        result = bytearray()
        for i in range(65536):
            byte = self.data[self.offset(rva + i)]
            if not byte:
                try:
                    return result.decode('ascii')
                except UnicodeDecodeError as e:
                    raise PEError('non-ASCII import name') from e
            result.append(byte)
        raise PEError('unterminated import name')

    def thunks(self, rva):
        result = []
        if not rva:
            raise PEError('missing import lookup table')
        for i in range(len(self.data) // self.width):
            value = self.unpack('<Q' if self.width == 8 else '<I', self.offset(rva + i * self.width, self.width))[0]
            if not value:
                return result
            if value & (1 << (self.bits - 1)):
                result.append({'ordinal': value & 0xffff})
            else:
                result.append({'name': self.string(value + 2)})
        raise PEError('unterminated thunk table')

    def imports(self, delay=False):
        index = 13 if delay else 1
        if len(self.directories) <= index:
            return []
        rva, size = self.directories[index]
        if not rva and not size:
            return []
        stride = 32 if delay else 20
        result = []
        for delta in range(0, size - stride + 1, stride):
            fields = self.unpack('<' + 'I' * (stride // 4), self.offset(rva + delta, stride))
            if not any(fields):
                return result
            if delay:
                attributes, name, _, iat, lookup, _, _, _ = fields
                if attributes & ~1:
                    raise PEError('unknown delay import attributes')
                if not attributes & 1:
                    # Legacy VA-based delay descriptors cannot represent >4GiB addresses.
                    name -= self.image_base
                    lookup = (lookup or iat) - self.image_base
                else:
                    lookup = lookup or iat
            else:
                lookup, _, _, name, iat = fields
                lookup = lookup or iat
            result.append({'module': self.string(name), 'symbols': self.thunks(lookup)})
        raise PEError('import directory lacks bounded terminator')


def inventory(path):
    data = path.read_bytes()
    pe = PE(data)
    normal, delay = pe.imports(), pe.imports(True)
    return {'schema_version': 1, 'binary': path.name, 'sha256': hashlib.sha256(data).hexdigest(),
            'size': len(data), 'machine': hex(pe.machine), 'bits': pe.bits,
            'imports': normal, 'delay_imports': delay,
            'limitations': ['Static import presence is not measured execution or exhaustive API coverage.',
                            'GetProcAddress targets, COM methods, syscalls and dependent DLL imports are not inferred.']}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    try:
        result = json.dumps(inventory(args.binary), indent=2) + '\n'
    except (PEError, OSError) as e:
        parser.exit(1, f'{e}\n')
    if args.output:
        args.output.write_text(result)
    else:
        print(result, end='')
