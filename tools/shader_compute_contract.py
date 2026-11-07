"""Fixed v1 finite DXIL compute observation; independent scalar expectation."""
import json
import struct
from .graphics_contract import unique_object


def expected_observation():
    count, salt = 19, 0x12345678
    words = []
    for i in range(count):
        source = i // 8 * 8 + 7 - i % 8
        a = (0xfffffff0 + source) & 0xffffffff if source < count else 0
        b = 3 * i + 7
        bits = struct.unpack('<I', struct.pack('<f', 4 * i + 22))[0]
        words.extend([(a + salt) & 0xffffffff, (b << (i & 7)) ^ salt, bits, b if i & 1 else a])
    words.extend([0xdeadbeef] * 20)
    return {'scenario': 'dxil-compute-v1', 'count': count, 'output_words': words}


def validate_observation(text):
    observed = json.loads(text, object_pairs_hook=unique_object)
    expected = expected_observation()
    if (not isinstance(observed, dict) or observed != expected
            or type(observed.get('count')) is not int
            or not isinstance(observed.get('output_words'), list)
            or any(type(word) is not int for word in observed['output_words'])):
        raise ValueError('compute observation differs from the fixed v1 contract')
    return observed


def compare_reports(reports, revision):
    if len(reports) != 2 or {r['backend'] for r in reports} != {'native D3D12 WARP', 'translated Metal'}:
        raise ValueError('need one native WARP and one translated Metal report')
    for report in reports:
        if (type(report['schema_version']) is not int or report['schema_version'] != 1
                or report['source_revision'] != revision
                or report['tracked_tree_clean'] is not True):
            raise ValueError('compute reports must belong to this clean revision')
        validate_observation(json.dumps(report['observation']))
        digest = report['shader_dxil_sha256']
        if not isinstance(digest, str) or len(digest) != 64 or any(c not in '0123456789abcdef' for c in digest):
            raise ValueError('invalid measured DXIL identity')
    if reports[0]['shader_dxil_sha256'] != reports[1]['shader_dxil_sha256']:
        raise ValueError('backends did not execute the same DXIL bytes')
    if reports[0]['observation'] != reports[1]['observation']:
        raise ValueError('native and translated compute words differ')
