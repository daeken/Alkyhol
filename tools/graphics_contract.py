"""Frozen v1 observation contract for the finite 65x4 RGBA8 render/copy scenario."""
import json


def expected_observation():
    pixels = bytes(channel for y in range(4) for x in range(65)
                   for channel in ((x % 2) * 255, (y % 2) * 255, ((x // 2 + y) % 2) * 255, 255)).hex()
    return {'scenario': 'rgba8-checker-copy-v1', 'width': 65, 'height': 4,
            'source_rgba8': pixels, 'destination_rgba8': pixels}


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f'duplicate observation field: {key}')
        value[key] = item
    return value


def validate_observation(text):
    observed = json.loads(text, object_pairs_hook=unique_object)
    expected = expected_observation()
    if (not isinstance(observed, dict) or observed != expected
            or any(type(observed[key]) is not type(value) for key, value in expected.items())):
        raise ValueError('graphics observation differs from the fixed v1 contract')
    return observed
