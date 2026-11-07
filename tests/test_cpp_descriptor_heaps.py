"""Synthetic handle protocol checks, separate from actual SDK/Metal execution."""
import copy
import json
import unittest
from tools.run_cpp_descriptor_heaps import SOURCE,compare_reports,semantic_observation,validate_observation
class DescriptorHeapReportTests(unittest.TestCase):
    def reports(self):
        rows=[]
        for kind in range(4):
            for visible in range(2 if kind<2 else 1):
                for node in [0,1]:rows.append({'desc':[kind,17,visible,1],'cpu_present':True,'gpu_present':bool(visible),'last_cpu_index':16,'last_gpu_index':16 if visible else 0})
        native={'schema_version':1,'backend':'native Windows SDK WARP descriptor heaps','source_revision':'current','tracked_tree_clean':True,'artifacts':{SOURCE:'source'},'observation':{'scenario':'descriptor-heaps-v1','strides':[32,16,32,32],'heaps':rows,'parent_lifetime':True}}
        host=copy.deepcopy(native);host['backend']='Rust restricted Metal COM descriptor heaps';host['observation']['strides']=[64]*4;return [native,host]
    def test_accepts_different_actual_driver_strides_but_preserves_raw_reports(self):
        reports=self.reports();compare_reports(reports,'current')
        self.assertNotEqual(reports[0]['observation']['strides'],reports[1]['observation']['strides'])
        self.assertEqual(semantic_observation(reports[0]['observation']),semantic_observation(reports[1]['observation']))
    def test_rejects_missing_invalid_fields_failed_ownership_and_zero_stride(self):
        for key,value in [('strides',[0]*4),('strides',[True]*4),('heaps',[]),('parent_lifetime',False),('scenario','other')]:
            observation=self.reports()[0]['observation'];observation[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
        for key,value in [('desc',[0]),('cpu_present',False),('gpu_present',0),('last_cpu_index',True)]:
            observation=self.reports()[0]['observation'];observation['heaps'][0][key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
    def test_requires_independent_clean_same_revision_and_source(self):
        for key,value in [('schema_version',0),('source_revision','old'),('tracked_tree_clean',False),('backend','native Windows SDK WARP descriptor heaps')]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        reports=self.reports();reports[1]['artifacts'][SOURCE]='changed'
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_never_normalizes_descriptor_fields_or_logical_handle_offsets(self):
        for field in ['desc','gpu_present','last_cpu_index','last_gpu_index']:
            reports=self.reports();row=reports[1]['observation']['heaps'][0]
            if field=='desc':row[field][3]=0
            elif field=='gpu_present':row[field]=not row[field]
            else:row[field]+=1
            with self.assertRaises(ValueError):compare_reports(reports,'current')
