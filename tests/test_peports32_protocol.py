"""Synthetic release/corpus protocol negatives; Windows execution is separate."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'tools'))
import fetch_peports32_binary as fetcher
import run_peports32_binary as runner
import compare_peports32_binary as comparator


class Peports32ProtocolTests(unittest.TestCase):
    def reports(self):
        _,wanted=runner.expected()
        native={'schema_version':1,'backend':runner.NATIVE,'program':runner.PROGRAM,'pe_sha256':runner.PE_SHA256,'executed_pe_sha256':runner.PE_SHA256,
                'upstream_source_sha256':runner.SOURCE_SHA256,'upstream_license_sha256':runner.LICENSE_SHA256,'input_sha256':runner.corpus_hashes(),
                'observations':wanted,'forced_rebase_observations':None,'revision':'a'*40,'tracked_tree_clean':True}
        host=copy.deepcopy(native);host.update(backend=runner.HOST,forced_rebase_observations=copy.deepcopy(wanted))
        return [native,host]

    def test_exact_unchanged_pe_inputs_imports_exports_errors_recovery_and_unicode_cases(self):
        records=self.reports();self.assertEqual(comparator.compare(records,reference=records[0]),16)
        cases,_=runner.expected();by_id={c['id']:c for c in cases}
        self.assertIn('Forwarded <KERNEL32.GetLastError>',by_id['exports']['stdout'])
        self.assertIn('ReadFile',by_id['recover-next-file']['stdout']);self.assertEqual(by_id['recover-next-file']['status'],1)
        self.assertIn('λ🌸',by_id['unicode-path']['args'][1]);self.assertEqual(set(runner.corpus_hashes()),{'pe/fixtures/peports/cases.json','fixture:probe.exe','fixture:exports.dll','fixture:bad.bin','fixture:truncated-stdin'})

    def test_each_case_missing_crashed_or_changed_raw_streams_is_rejected(self):
        original=self.reports()
        for case in original[0]['observations']:
            for field,value in [('status',-11),('status',False),('stdout_hex','00'),('stderr_hex','00')]:
                reports=copy.deepcopy(original);reports[1]['observations'][case][field]=value
                with self.assertRaises(ValueError):comparator.compare(reports,reference=original[0])
            reports=copy.deepcopy(original);del reports[1]['observations'][case]
            with self.assertRaises(ValueError):comparator.compare(reports,reference=original[0])

    def test_release_fixture_corpus_execution_revision_and_forced_rebase_identity_are_required(self):
        for key,value in [('schema_version',True),('tracked_tree_clean',False),('revision','b'*40),('backend',runner.NATIVE),('program','fake'),('pe_sha256','c'*64),('executed_pe_sha256','c'*64),('upstream_source_sha256','c'*64),('upstream_license_sha256','c'*64),('input_sha256',{}),('forced_rebase_observations',None)]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):comparator.compare(reports,reference=self.reports()[0])
        reports=self.reports();reports[1]['forced_rebase_observations']['help']['status']=False
        with self.assertRaises(ValueError):comparator.compare(reports,reference=self.reports()[0])

    def test_only_exact_verified_entry_prefix_is_removed_and_real_stdio_is_observed(self):
        prefix=b'PE32: 6 sections, 10 imports, 239 HIGHLOW fixups\nchecked i386 mapping; relocated=true\n'
        restored=b'host segment selectors restored; owned FS descriptor cleared\n'
        run=subprocess.CompletedProcess([],1,b'raw\0data',prefix+b'peports: original error\n'+restored)
        self.assertEqual(runner.decode(run,True,True)['stderr_hex'],b'peports: original error\n'.hex())
        for stderr in [b'',b'noise\n'+prefix+restored,prefix.replace(b'true',b'false')+restored,prefix]:
            run.stderr=stderr
            with self.assertRaises(ValueError):runner.decode(run,True,True)
        case=runner.expected()[0][0];run=subprocess.CompletedProcess([],case['status'],case['stdout'].encode(),case['stderr'].encode())
        with mock.patch.object(runner,'fixture_bytes',return_value={'probe.exe':b'PE data'}),mock.patch.object(runner,'expected',return_value=([case],{case['id']:runner.decode(run)})),mock.patch.object(runner.subprocess,'run',return_value=run) as execute:
            self.assertEqual(len(runner.observe(Path('/trusted/released.exe'),False)),1);self.assertEqual(execute.call_args.kwargs['input'],b'')
            execute.return_value=subprocess.CompletedProcess([],-11,b'',b'crash')
            with self.assertRaises(ValueError):runner.observe(Path('/trusted/released.exe'),False)

    def test_archive_and_frozen_input_pe_are_checked_before_use(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);archive=root/'local/pe/off-the-shelf/w64devkit-x86-2.11.0.7z.exe';archive.parent.mkdir(parents=True);archive.write_bytes(b'corrupt SFX')
            with mock.patch.object(fetcher,'ROOT',root),mock.patch.object(fetcher.platform,'system',return_value='Darwin'),mock.patch.object(fetcher.subprocess,'check_output') as extract,mock.patch.object(fetcher.urllib.request,'urlopen') as network:
                with self.assertRaises(ValueError):fetcher.fetch(root/'output')
                extract.assert_not_called();network.assert_not_called()
            probe=root/'local/pe/fixture/contract.exe';probe.parent.mkdir(parents=True);probe.write_bytes(b'changed PE')
            exports=root/'local/pe/export-fixture/alkyhol_exports.dll';exports.parent.mkdir();exports.write_bytes(b'changed DLL')
            with mock.patch.object(runner,'ROOT',root):
                with self.assertRaises(ValueError):runner.fixture_bytes()

    def test_native_record_is_required_unless_explicitly_supplied_for_first_capture(self):
        reports=self.reports()
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch.object(comparator,'REFERENCE',Path(directory)/'missing.json'):
                with self.assertRaises(FileNotFoundError):comparator.compare(reports)
        self.assertEqual(comparator.compare(reports,reference=reports[0]),16)

    def test_required_frozen_native_record_rejects_joint_corpus_drift(self):
        native=json.loads(runner.REFERENCE.read_text(encoding='utf-8'))
        host=copy.deepcopy(native);host.update(backend=runner.HOST,forced_rebase_observations=copy.deepcopy(native['observations']))
        self.assertEqual(comparator.compare([native,host]),16)
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch.object(comparator,'REFERENCE',Path(directory)/'missing.json'):
                with self.assertRaises(FileNotFoundError):comparator.compare([native,host])
            changed=Path(directory)/'changed.json';changed.write_bytes(runner.CORPUS.read_bytes()+b' ')
            with mock.patch.object(runner,'CORPUS',changed):
                for record in [native,host]:record['input_sha256']=runner.corpus_hashes()
                with self.assertRaisesRegex(ValueError,'frozen native evidence'):comparator.compare([native,host])


if __name__=='__main__':unittest.main()
