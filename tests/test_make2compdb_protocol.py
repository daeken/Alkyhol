"""Synthetic protocol negatives; actual released-PE/native evidence is separate."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
import fetch_make2compdb_binary as fetcher
import run_make2compdb_binary as runner
import compare_make2compdb_binary as comparator
from compare_make2compdb_binary import compare


class Make2CompdbProtocolTests(unittest.TestCase):
    def reports(self):
        _,wanted=runner.expected()
        native={'schema_version':1,'backend':runner.NATIVE,'program':runner.PROGRAM,'pe_sha256':runner.PE_SHA256,
                'executed_pe_sha256':runner.PE_SHA256,'upstream_source_sha256':runner.SOURCE_SHA256,
                'upstream_license_sha256':runner.LICENSE_SHA256,'input_sha256':{'pe/fixtures/make2compdb/cases.json':hashlib.sha256(runner.CORPUS.read_bytes()).hexdigest()},
                'observations':wanted,'forced_rebase_observations':None,'revision':'a'*40,'tracked_tree_clean':True}
        host=copy.deepcopy(native);host.update(backend=runner.HOST,forced_rebase_observations=copy.deepcopy(wanted))
        return [native,host]

    def test_all_exact_raw_cases_include_real_work_unicode_quoted_cli_and_chunked_stdin(self):
        records=self.reports();self.assertEqual(compare(records,reference=records[0]),16)
        cases,_=runner.expected();chunk=next(c for c in cases if c['id']=='chunked-input')
        self.assertGreater(len(chunk['stdin'].encode()),16384)
        self.assertEqual(len(json.loads(next(c['stdout'] for c in cases if c['id']=='nested'))),3)
        self.assertIn('λ🌸',next(c['stdout'] for c in cases if c['id']=='unicode'))

    def test_every_case_crash_missing_and_changed_status_stdout_stderr_is_rejected(self):
        records=self.reports()
        for key in records[0]['observations']:
            for field,value in [('status',-11),('status',False),('stdout_hex','00'),('stderr_hex','00')]:
                changed=copy.deepcopy(records);changed[1]['observations'][key][field]=value
                with self.assertRaises(ValueError):compare(changed,reference=records[0])
            changed=copy.deepcopy(records);del changed[1]['observations'][key]
            with self.assertRaises(ValueError):compare(changed,reference=records[0])

    def test_release_execution_source_corpus_revision_and_backend_identity_are_required(self):
        for field,value in [('schema_version',True),('tracked_tree_clean',False),('revision','b'*40),('backend',runner.NATIVE),('program','pretend'),
            ('pe_sha256','c'*64),('executed_pe_sha256','c'*64),('upstream_source_sha256','c'*64),('upstream_license_sha256','c'*64),('input_sha256',{}),('forced_rebase_observations',None)]:
            records=self.reports();records[1][field]=value
            with self.assertRaises(ValueError):compare(records,reference=self.reports()[0])
        records=self.reports();records[1]['forced_rebase_observations']['empty']['status']=False
        with self.assertRaises(ValueError):compare(records,reference=self.reports()[0])

    def test_only_the_exact_loader_prefix_is_removed_and_rebasing_must_have_happened(self):
        prefix=b'PE64: 6 sections, 10 named imports, 95 DIR64 fixups\nentering Win64 code; relocated=true\n'
        run=subprocess.CompletedProcess([],1,b'raw\0stdout',prefix+b'Unknown CLI arg: x\n')
        self.assertEqual(runner.decode(run,True,True),{'status':1,'stdout_hex':run.stdout.hex(),'stderr_hex':b'Unknown CLI arg: x\n'.hex()})
        for stderr in [b'',b'pretend success\n',prefix.replace(b'true',b'false'),b'noise\n'+prefix]:
            run.stderr=stderr
            with self.assertRaises(ValueError):runner.decode(run,True,True)

    def test_observer_passes_exact_stdin_and_rejects_an_actual_failed_process(self):
        case=runner.expected()[0][0]
        run=subprocess.CompletedProcess([],case['status'],case['stdout'].encode(),case['stderr'].encode())
        with mock.patch.object(runner,'expected',return_value=([case],{case['id']:runner.decode(run)})),mock.patch.object(runner.subprocess,'run',return_value=run) as execute:
            self.assertEqual(len(runner.observe(Path('/trusted/released.exe'),False)),1)
            self.assertEqual(execute.call_args.kwargs['input'],case['stdin'].encode())
            self.assertEqual(execute.call_args.kwargs['cwd'].name,'build space λ🌸')
            execute.return_value=subprocess.CompletedProcess([],-11,b'',b'crash')
            with self.assertRaises(ValueError):runner.observe(Path('/trusted/released.exe'),False)

    def test_unverified_archive_is_never_extracted_or_executed(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);archive=root/'local/pe/off-the-shelf/w64devkit-x64-2.11.0.7z.exe';archive.parent.mkdir(parents=True);archive.write_bytes(b'corrupt SFX')
            with mock.patch.object(fetcher,'ROOT',root),mock.patch.object(fetcher.platform,'system',return_value='Darwin'),mock.patch.object(fetcher.subprocess,'check_output') as extract,mock.patch.object(fetcher.urllib.request,'urlopen') as network:
                with self.assertRaises(ValueError):fetcher.fetch(root/'output')
                extract.assert_not_called();network.assert_not_called()

    def test_frozen_native_record_is_required_and_joint_corpus_drift_cannot_replace_it(self):
        native=json.loads(runner.REFERENCE.read_text(encoding='utf-8'))
        host=copy.deepcopy(native);host.update(backend=runner.HOST,forced_rebase_observations=copy.deepcopy(native['observations']))
        self.assertEqual(compare([native,host]),16)
        with tempfile.TemporaryDirectory() as directory:
            missing=Path(directory)/'missing.json'
            with mock.patch.object(comparator,'REFERENCE',missing):
                with self.assertRaises(FileNotFoundError):compare([native,host])
            changed=Path(directory)/'changed.json';changed.write_bytes(runner.CORPUS.read_bytes()+b' ')
            digest=hashlib.sha256(changed.read_bytes()).hexdigest()
            for record in [native,host]:record['input_sha256']={'pe/fixtures/make2compdb/cases.json':digest}
            with mock.patch.object(comparator,'CORPUS',changed):
                with self.assertRaisesRegex(ValueError,'frozen native evidence'):compare([native,host])


if __name__=='__main__':unittest.main()
