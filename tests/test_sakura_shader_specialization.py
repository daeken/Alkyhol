import unittest
from tools.prepare_sakura_shaders import functions, mask_comments_and_strings, prepare


class SakuraShaderSpecializationTests(unittest.TestCase):
    def test_exact_spans_ignore_comment_and_string_braces_and_keep_overloads(self):
        source = ('/*\nfloat4 F() { bad; }\n*/\n'
                  '[shader("miss")]\r\nvoid F(float a) { /* } */ if(a) { a=1; } }\r\n'
                  'float F(float a,float b) { const char* s="}\\\"{"; return a+b; }\n')
        result = functions(source, "F")
        self.assertEqual(len(result), 2)
        self.assertTrue(result[0].startswith('[shader("miss")]\r\n'))
        self.assertTrue(result[0].endswith('a=1; } }'))
        self.assertIn('return a+b;', result[1])
        self.assertEqual(len(mask_comments_and_strings(source)), len(source))

    def test_missing_unbalanced_and_unterminated_input_fails(self):
        for source in ['void Other() {}', 'void F() {', 'void F();', '/*', '"bad']:
            with self.assertRaises(ValueError):
                functions(source, 'F')

    def test_pinned_extractions_are_reproducible_without_body_edits(self):
        prepare(check=True)


if __name__ == '__main__':
    unittest.main()
