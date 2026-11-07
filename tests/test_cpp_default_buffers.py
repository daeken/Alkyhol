"""Synthetic report gates; real Windows/Metal execution supplies observations."""
import copy
import json
import unittest
from tools.run_cpp_default_buffers import SOURCE,compare_reports,validate_observation
class DefaultBufferReportTests(unittest.TestCase):
    def reports(self):
        observation={'scenario':'default-buffer-transitions-v1','map_hr':0x80070057,
                     'heap':[1,0,0,1,1],'heap_flags':192,'desc':[1,65536,1024,1,1,1,0,1,0,1,0],
                     'fences':[1,2,3],'buffers':['a5'*1024]*4}
        native={'schema_version':1,'backend':'native Windows SDK WARP DEFAULT buffers',
                'source_revision':'current','tracked_tree_clean':True,'artifacts':{SOURCE:'source'},'observation':observation}
        host=copy.deepcopy(native);host['backend']='Rust restricted Metal COM DEFAULT buffers';return [native,host]
    def test_preserves_native_fields_without_normalizing_or_inventing_them(self):
        reports=self.reports();compare_reports(reports,'current')
        observation=reports[0]['observation'];observation['map_hr']=1;observation['desc'][1]=0
        self.assertEqual(validate_observation(json.dumps(observation)),observation)
    def test_rejects_incomplete_invalid_or_wrong_schema(self):
        for name,value in [('scenario','other'),('map_hr',True),('heap_flags',-1),('heap',[0]*4),('desc',[0]*10),('fences',[False]*3),('buffers',['00']),('buffers',['zz'*1024]*4)]:
            report=self.reports()[0]['observation'];report[name]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(report))
    def test_requires_independent_clean_matching_source_revision_and_backend(self):
        for name,value in [('schema_version',0),('source_revision','old'),('tracked_tree_clean',False),('backend','native Windows SDK WARP DEFAULT buffers')]:
            reports=self.reports();reports[1][name]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        reports=self.reports();reports[1]['artifacts'][SOURCE]='changed'
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_each_returned_word_and_every_gpu_or_guard_byte_is_required(self):
        for field in ['map_hr','heap_flags','heap','desc','fences']:
            reports=self.reports();value=reports[1]['observation'][field]
            if isinstance(value,list):value[-1]+=1
            else:reports[1]['observation'][field]+=1
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        for buffer in range(4):
            for byte in range(1024):
                reports=self.reports();text=reports[1]['observation']['buffers'][buffer]
                reports[1]['observation']['buffers'][buffer]=text[:byte*2]+'00'+text[byte*2+2:]
                with self.assertRaises(ValueError):compare_reports(reports,'current')
