import json
import unittest
import subprocess
import sys
import tempfile
from tools.graphics_contract import expected_observation, validate_observation
from tools.run_graphics_contract import execute_probe


class GraphicsContractTests(unittest.TestCase):
    def test_accepts_full_exact_source_and_destination_pixels(self):
        observation = expected_observation()
        self.assertEqual(len(bytes.fromhex(observation['source_rgba8'])), 65 * 4 * 4)
        self.assertEqual(validate_observation(json.dumps(observation)), observation)
        pixels = bytes.fromhex(observation['source_rgba8'])
        self.assertEqual(pixels[:16], bytes([0, 0, 0, 255, 255, 0, 0, 255, 0, 0, 255, 255, 255, 0, 255, 255]))
        self.assertEqual(pixels[65 * 4:65 * 4 + 4], bytes([0, 255, 255, 255]))

    def test_rejects_any_single_channel_change_on_either_surface(self):
        for field in ['source_rgba8', 'destination_rgba8']:
            original = bytearray.fromhex(expected_observation()[field])
            for offset in range(len(original)):
                observation = expected_observation()
                changed = original.copy(); changed[offset] ^= 1
                observation[field] = changed.hex()
                with self.subTest(field=field, offset=offset), self.assertRaises(ValueError):
                    validate_observation(json.dumps(observation))

    def test_rejects_missing_extra_wrong_dimensions_types_and_truncated_pixels(self):
        baseline = expected_observation()
        variants = []
        for key in baseline:
            missing = baseline.copy(); del missing[key]; variants.append(missing)
        variants.extend([
            {**baseline, 'extra': True}, {**baseline, 'width': 65.0}, {**baseline, 'height': 3},
            {**baseline, 'source_rgba8': baseline['source_rgba8'][:-2]},
            {**baseline, 'destination_rgba8': baseline['destination_rgba8'].upper()},
            {**baseline, 'scenario': 'different'}, [], None,
        ])
        for value in variants:
            with self.subTest(value_type=type(value).__name__), self.assertRaises(ValueError):
                validate_observation(json.dumps(value))

    def test_rejects_duplicates_multiple_observations_or_malformed_json(self):
        valid = json.dumps(expected_observation())
        for text in ['not JSON', valid + valid, valid[:-1] + ',"width":65}']:
            with self.assertRaises(ValueError): validate_observation(text)


class GraphicsProcessTests(unittest.TestCase):
    """Synthetic process fault tests only; these are not GPU execution evidence."""
    def command(self, adapter, extra=''):
        return [sys.executable, '-c', 'import sys; print(' + repr(json.dumps(expected_observation())) + '); sys.stderr.write(' + repr(adapter) + '); ' + extra]

    def test_accepts_exact_pixels_with_one_matching_backend_identity(self):
        adapter = 'adapter: synthetic device_type: IntegratedGpu backend: Metal\n'
        with tempfile.TemporaryDirectory() as directory:
            observed, identity, _ = execute_probe(self.command(adapter), directory, 'Alkyhol direct Metal')
        self.assertEqual(observed, expected_observation())
        self.assertEqual(identity, adapter.strip())

    def test_missing_duplicate_wrong_or_silent_software_adapter_cannot_pass(self):
        for adapter, backend in [
            ('', 'Alkyhol direct Metal'),
            ('adapter: backend: Metal\nadapter: backend: Metal\n', 'Alkyhol direct Metal'),
            ('adapter: backend: Vulkan\n', 'Alkyhol direct Metal'),
            ('adapter: device_type: IntegratedGpu backend: Vulkan\n', 'Alkyhol direct Metal'),
            ('adapter: device_type: Cpu backend: Vulkan\n', 'Alkyhol Vulkan'),
            ('adapter: device_type: Cpu backend: Metal\n', 'Alkyhol direct Metal'),
            ('adapter: backend: Metal\n', 'native D3D12 WARP'),
        ]:
            with self.subTest(adapter=adapter, backend=backend), tempfile.TemporaryDirectory() as directory, self.assertRaises((RuntimeError, ValueError)):
                execute_probe(self.command(adapter), directory, backend)

    def test_nonzero_exit_cannot_pass_despite_valid_pixels_and_adapter(self):
        with tempfile.TemporaryDirectory() as directory, self.assertRaises(subprocess.CalledProcessError):
            execute_probe(self.command('adapter: backend: Metal\n', 'raise SystemExit(1)'), directory, 'Alkyhol direct Metal')

    def test_timeout_cannot_pass_despite_already_printed_valid_pixels(self):
        with tempfile.TemporaryDirectory() as directory, self.assertRaises(subprocess.TimeoutExpired):
            execute_probe(self.command('adapter: backend: Metal\n', 'sys.stdout.flush(); import time; time.sleep(30)'), directory, 'Alkyhol direct Metal', timeout=0.1)


if __name__ == '__main__':
    unittest.main()
