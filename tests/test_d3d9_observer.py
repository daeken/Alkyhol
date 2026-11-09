"""Comparator regression units; synthetic records are not rendering evidence."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"tools"))
from observe_d3d9_create_device import triangle_pixels,compare_triangle_pixels,shader_words
import observe_d3d9_create_device as observer
from unittest.mock import patch

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
    def test_font_record_retains_alpha_in_both_complete_frames(self):
        words=[0x39534844,15]+[1]*21
        for frame,color in [(0,0x99000000),(1,0x11335577)]:
            pixels=[0xff0000ff]*1024;pixels[20*32+22]=color
            pixels[20*32+10]=color if frame==0 else 0xff0000ff
            words += [0xfeed0000+frame]+[1]*8+pixels+[1]
        words += [0xfeedffff]+[1]*7+[2100]
        with patch.object(observer,'FONTS',True):
            self.assertEqual(shader_words(words),words)
            for at,value in [(23+9+20*32+22,0x98000000),(1057+9+20*32+22,0x00335577)]:
                changed=words.copy();changed[at]=value
                with self.assertRaises(ValueError):shader_words(changed)
    def test_texture_record_checks_srgb_and_its_separate_metadata_extent(self):
        words=[0x39534844,2]+[1]*34
        for frame,color in [(0,0x808080),(1,0x373737)]:
            pixels=[0xff]*1024;pixels[20*32+22]=color
            pixels[20*32+10]=color if frame==0 else 0xff
            words += [0xfeed0000+frame]+[1]*10+pixels+[1]
        words += [0xfeedffff]+[1]*7+[2117]
        self.assertEqual(shader_words(words,True),words)
        words[1083+20*32+22]=0x808080
        with self.assertRaises(ValueError):shader_words(words,True)
    def test_model2_record_requires_bounded_profile_and_long_shader_color(self):
        words=[0x39534844,3]+[1]*21
        for frame in range(2):
            pixels=[0xff]*1024;pixels[20*32+22]=0xa347eb
            pixels[20*32+10]=0xa347eb if frame==0 else 0xff
            words += [0xfeed0000+frame]+[1]*8+pixels+[1]
        words += [0xfeedffff]+[1]*7+[2100]
        self.assertEqual(shader_words(words,model2=True),words)
        words[2]=0
        with self.assertRaises(ValueError):shader_words(words,model2=True)
    def test_render_state_record_requires_actual_blending_and_write_mask(self):
        words=[0x39534844,4]+[1]*42
        for frame,color in [(0,0x4040bf),(1,0x1c00ff)]:
            pixels=[0xff]*1024;pixels[20*32+22]=color
            pixels[20*32+10]=color if frame==0 else 0xff
            words += [0xfeed0000+frame]+[1]*14+pixels+[1]
        words += [0xfeedffff]+[1]*7+[2133]
        self.assertEqual(shader_words(words,True,states=True),words)
        words[44+15+20*32+22]=0x808080
        with self.assertRaises(ValueError):shader_words(words,True,states=True)
    def test_float_bias_requires_occlusion_and_both_complete_frames(self):
        words=self.record();words[1]=20
        words[31:31+1024]=[0xff]*1024
        with patch.object(observer,'BIAS',True):
            self.assertEqual(shader_words(words),words)
            for index,value in [(31+1023,0xff0000),(1065+20*32+22,0xff),(1065+20*32+10,0xff0000),(2,0)]:
                changed=words.copy();changed[index]=value
                with self.assertRaises(ValueError):shader_words(changed)
            for changed in [words[:-1],words+[0]]:
                with self.assertRaises(ValueError):shader_words(changed)

    def test_alpha_test_rejection_requires_both_frames_and_exact_rgb(self):
        words=self.record();words[1]=24;words[31:31+1024]=[0xff]*1024
        with patch.object(observer,'ALPHA_TEST',True):
            self.assertEqual(shader_words(words),words)
            for index,value in [(31+1023,0xff0000),(1065+20*32+22,0xff),(1065+1023,0x123456),(2,0)]:
                changed=words.copy();changed[index]=value
                with self.assertRaises(ValueError):shader_words(changed)

    def test_occlusion_counter_report_requires_overlap_and_hidden_samples(self):
        words=self.record();words[1]=21;words[20:20]=[1,552,276,0];words[-1]=2103
        with patch.object(observer,'OCCLUSION',True):
            self.assertEqual(shader_words(words),words)
            for index,value in [(20,0),(21,2),(22,1),(23,1),(21,0),(22,0),(22,1025)]:
                changed=words.copy();changed[index]=value
                with self.assertRaises(ValueError):shader_words(changed)
            for changed in [words[:-1],words+[0]]:
                with self.assertRaises(ValueError):shader_words(changed)

    def test_user_draw_requires_all_four_complete_frames_and_binding_assertions(self):
        words=[0x39534844,23]+[1]*18+[0xff0000]*1024+[1]*3
        for frame in range(4):
            pixels=[0xff]*1024;pixels[20*32+22]=0xff0000
            pixels[20*32+10]=0xff0000 if frame==0 else 0xff
            words += [0xfeed0000+frame]+[1]*8+pixels+[1]
        words += [0xfeedffff]+[1]*7+[5192]
        with patch.object(observer,'USER_DRAW',True):
            self.assertEqual(shader_words(words),words)
            for frame in range(4):
                for index,value in [(1047+1034*frame+5,0),(1047+1034*frame+9+20*32+22,0xff),(1047+1034*frame+9+1023,0xdeadbeef)]:
                    changed=words.copy();changed[index]=value
                    with self.assertRaises(ValueError):shader_words(changed)
            changed=words.copy();changed[20+1023]=0xff
            with self.assertRaises(ValueError):shader_words(changed)
            for changed in [words[:-1],words+[0],words[:1047]+words[1047+1034:]]:
                with self.assertRaises(ValueError):shader_words(changed)

    def test_exact_frames_and_stack_extent(self):
        words=self.record();self.assertEqual(shader_words(words),words)
        for changed in [words[:-1],words+[0]]:
            with self.assertRaises(ValueError):shader_words(changed)
    def test_failures_or_missing_constant_effect_cannot_pass(self):
        for index,value in [(9,0),(25,0),(31,0),(1065+20*32+10,0xff0000)]:
            words=self.record();words[index]=value
            with self.assertRaises(ValueError):shader_words(words)

if __name__=="__main__":unittest.main()
