"""Synthetic report gates; real Windows/Metal execution supplies observations."""
import copy
import hashlib
import json
import unittest
from tools.run_cpp_default_buffers import ROOT,SOURCE,compare_reports,validate_observation
class DefaultBufferReportTests(unittest.TestCase):
    def reports(self):
        observation=json.loads((ROOT/'cpp/reference/default-buffer-transitions-windows-v1.json').read_text())['observation']
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
                reports[1]['observation']['buffers'][buffer]=text[:byte*2]+format(int(text[byte*2:byte*2+2],16)^1,'02x')+text[byte*2+2:]
                with self.assertRaises(ValueError):compare_reports(reports,'current')

    def test_live_backends_cannot_jointly_replace_frozen_native_values(self):
        reports=self.reports()
        for report in reports:report['observation']['heap_flags']=0
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_frozen_native_source_execution_and_observation_hash_are_preserved(self):
        report=json.loads((ROOT/'cpp/reference/default-buffer-transitions-windows-v1.json').read_text())
        self.assertEqual(report['backend'],'native Windows SDK WARP DEFAULT buffers')
        self.assertIs(report['tracked_tree_clean'],True)
        self.assertEqual(report['source_revision'],'4c09ebcb051f9a2fa8aeb29daa636d2b714f5f8c')
        self.assertEqual(report['artifacts'][SOURCE],hashlib.sha256((ROOT/SOURCE).read_bytes()).hexdigest())
        self.assertEqual(hashlib.sha256(json.dumps(report['observation'],sort_keys=True,separators=(',',':')).encode()).hexdigest(),'98868d79e4b04b6e2e76d35c4c06097f59b031622160f7d5860447ed4d081586')
