"""Comparator regression units; synthetic records are not rendering evidence."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools"))
from observe_d3d9_create_device import triangle_pixels,compare_triangle_pixels,shader_words

class TriangleComparatorTests(unittest.TestCase):
    def test_metadata_and_footer_are_excluded_from_the_complete_pixel_region(self):
        pixels=[0xff]*90000;pixels[-1]=0x123456
        words=[0x44394433,4,0]+[1]*20+pixels+[1]*8+[90032]
        self.assertEqual(triangle_pixels(words),pixels)
        self.assertEqual(len(triangle_pixels(words)),90000)
    def test_one_lsb_is_allowed_but_two_is_rejected(self):
        first=[0x123456]*90000;second=first.copy();second[511]=0x133555
        compare_triangle_pixels(first,second)
        second[511]=0x143456
        with self.assertRaises(ValueError):compare_triangle_pixels(first,second)
    def test_coverage_must_match_even_for_a_one_lsb_color_difference(self):
        first=[0xff]*90000;second=first.copy();second[89999]=0x1ff
        with self.assertRaises(ValueError):compare_triangle_pixels(first,second)
    def test_truncated_or_extra_data_cannot_be_lost_by_zip(self):
        for first,second in [([0]*89999,[0]*90000),([0]*90000,[0]*90001)]:
            with self.assertRaises(ValueError):compare_triangle_pixels(first,second)
        with self.assertRaises(ValueError):triangle_pixels([0]*90031)

class LegacyShaderComparatorTests(unittest.TestCase):
    def record(self):
        words=[0x39534844,1]+[1]*20
        for frame in range(2):
            pixels=[0xff]*1024;pixels[20*32+22]=0xff0000
            pixels[20*32+10]=0xff0000 if frame==0 else 0xff
            words += [0xfeed0000+frame]+[1]*8+pixels+[1]
        return words+[0xfeedffff]+[1]*7+[2099]
    def test_exact_frames_and_stack_extent(self):
        words=self.record();self.assertEqual(shader_words(words),words)
        for changed in [words[:-1],words+[0]]:
            with self.assertRaises(ValueError):shader_words(changed)
    def test_failures_or_missing_constant_effect_cannot_pass(self):
        for index,value in [(9,0),(25,0),(31,0),(1065+20*32+10,0xff0000)]:
            words=self.record();words[index]=value
            with self.assertRaises(ValueError):shader_words(words)

if __name__=="__main__":unittest.main()
