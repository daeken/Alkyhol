"""Comparator regression units; synthetic records are not rendering evidence."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools"))
from observe_d3d9_create_device import triangle_pixels,compare_triangle_pixels

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

if __name__=="__main__":unittest.main()
