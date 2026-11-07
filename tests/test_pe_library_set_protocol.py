"""Synthetic bundle evidence protocol tests; actual DLL runs live in PE CI."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from build_pe_library_set import ORDER, expected_identity
from run_pe_library_set import EXPECTED, EXPECTED_TRACE, check_execution
from compare_pe_library_set import compare, NATIVE, HOST


class LibrarySetProtocolTests(unittest.TestCase):
    def reports(self):
        native = {'schema_version':1, 'backend':NATIVE, 'revision':'a'*40, 'tracked_tree_clean':True,
                  **expected_identity(), 'provided_modules':copy.deepcopy(ORDER),
                  'common':{'results':copy.deepcopy(EXPECTED),'trace_hex':EXPECTED_TRACE.hex()},
                  'relocated':None,'rebased_common':None,'rebased_relocated':None}
        host = copy.deepcopy(native)
        host.update(backend=HOST, relocated=[True,True,False], rebased_relocated=[True]*3, rebased_common=copy.deepcopy(host['common']))
        return [native,host]

    def execution(self,windows=False):
        return subprocess.CompletedProcess([],0,EXPECTED_TRACE,json.dumps({'common':EXPECTED,'relocated':None if windows else [True]*3}).encode())

    def test_exact_shared_results_and_native_rebase_separation(self):
        for windows in [False,True]:
            common,flags = check_execution(self.execution(windows),windows,rebase=not windows)
            self.assertEqual(common,self.reports()[0]['common'])
            self.assertEqual(flags,None if windows else [True]*3)
        reports = self.reports()
        self.assertEqual(compare(reports,reference=reports[0]),48)

    def test_crash_duplicate_missing_reordered_and_changed_notifications_fail(self):
        for trace in [b'',EXPECTED_TRACE[:-1],EXPECTED_TRACE+EXPECTED_TRACE[:24],EXPECTED_TRACE[24:]+EXPECTED_TRACE[:24],b'X'+EXPECTED_TRACE[1:]]:
            run = self.execution()
            run.stdout = trace
            with self.assertRaises(ValueError): check_execution(run)
        run = self.execution()
        run.returncode = -11
        with self.assertRaises(ValueError): check_execution(run)

    def test_wrong_shared_target_calls_and_incomplete_rebase_fail(self):
        for section,key in [('dependent','evaluation'),('ordinal','initial'),('ordinal','data_matches_ordinal')]:
            run = self.execution()
            result = json.loads(run.stderr)
            result['common'][section][key] = 0
            run.stderr = json.dumps(result).encode()
            with self.assertRaises(ValueError): check_execution(run)
        run = self.execution()
        result = json.loads(run.stderr)
        result['common']['shared_leaf_targets'] = False
        run.stderr = json.dumps(result).encode()
        with self.assertRaises(ValueError): check_execution(run)
        for flags in [None,[],[True,True],[True,False,True],[1,True,True]]:
            run = self.execution()
            result = json.loads(run.stderr)
            result['relocated'] = flags
            run.stderr = json.dumps(result).encode()
            with self.assertRaises(ValueError): check_execution(run,rebase=True)

    def test_dirty_backend_revision_and_joint_frozen_identity_drift_fail(self):
        for field,value in [('tracked_tree_clean',False),('backend',NATIVE),('revision','d'*40),('provided_modules',[]),('sources',{}),('binaries',{})]:
            reports = self.reports()
            reports[1][field] = value
            with self.assertRaises(ValueError): compare(reports,reference=self.reports()[0])
        reports = self.reports()
        for r in reports: r['binaries']['alkyhol_dependent.dll'] = 'e'*64
        with self.assertRaises(ValueError): compare(reports,reference=self.reports()[0])

    def test_missing_forced_rebase_and_invented_native_observations_fail(self):
        for field,value in [('rebased_common',None),('rebased_relocated',[True,True,False])]:
            reports = self.reports()
            reports[1][field] = value
            with self.assertRaises(ValueError): compare(reports,reference=self.reports()[0])
        reports = self.reports()
        reports[0]['relocated'] = [True]*3
        with self.assertRaises(ValueError): compare(reports,reference=self.reports()[0])


if __name__ == '__main__': unittest.main()
