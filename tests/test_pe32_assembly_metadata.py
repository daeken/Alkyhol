"""Owned XML/input checks only; not native binding or game evidence."""
import base64
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_assembly_metadata as observer

class AssemblyMetadataInputsTests(unittest.TestCase):
    def test_cases_isolate_private_version_and_integrity_metadata(self):
        ns={'a':'urn:schemas-microsoft-com:asm.v1','v2':'urn:schemas-microsoft-com:asm.v2','d':'http://www.w3.org/2000/09/xmldsig#'}
        original=(observer.SOURCE/'ctx-assembly.manifest').read_bytes()
        cases=observer.manifests()
        newer=ET.fromstring(cases['newer-version'])
        self.assertEqual(newer.find('a:assemblyIdentity',ns).get('version'),'2.0.0.0')
        self.assertIsNone(newer.find('a:file',ns).get('hash'))
        for name,version in [('hash-mismatch','1.0.0.0'),('token-newer-hash','2.0.0.0')]:
            root=ET.fromstring(cases[name]);identity=root.find('a:assemblyIdentity',ns)
            self.assertEqual(identity.get('version'),version)
            self.assertIsNotNone(root.find('a:noInheritable',ns))
            file=root.find('a:file',ns)
            self.assertEqual(file.get('hash'),'0'*40)
            self.assertEqual(base64.b64decode(file.find('v2:hash/d:DigestValue',ns).text),bytes(20))
        request=ET.fromstring(cases['token-importer']).find('a:dependency/a:dependentAssembly/a:assemblyIdentity',ns)
        provider=ET.fromstring(cases['token-newer-hash']).find('a:assemblyIdentity',ns)
        self.assertEqual(request.get('publicKeyToken'),provider.get('publicKeyToken'))
        self.assertEqual(request.get('version'),'1.0.0.0')
        self.assertEqual((observer.SOURCE/'ctx-assembly.manifest').read_bytes(),original)

    def test_hash_inventory_cannot_legitimize_an_undeclared_xml_case(self):
        with tempfile.TemporaryDirectory() as temporary,patch.object(observer,'verify_base',return_value={'owned':'synthetic transport only'}):
            directory=Path(temporary)
            for name,xml in observer.manifests().items():(directory/f'ctx-meta-{name}.manifest').write_bytes(xml.encode())
            dll=directory/'ctx-meta-token-importer.dll';dll.write_bytes(b'owned synthetic transport only')
            manifest=directory/'ctx-meta-newer-version.manifest';inventory=directory/'ctx-meta-inputs.json'
            inventory.write_text(json.dumps(observer.inputs(directory)))
            with patch.object(observer,'frozen_metadata',return_value=[{'inputs':observer.inputs(directory)}]):observer.verify(directory)
            manifest.write_bytes(manifest.read_bytes().replace(b'2.0.0.0',b'9.0.0.0'))
            inventory.write_text(json.dumps(observer.inputs(directory)))
            with self.assertRaisesRegex(ValueError,'declared owned case'):observer.verify(directory)
            manifest.write_bytes(observer.manifests()['newer-version'].encode())
            inventory.write_text(json.dumps(observer.inputs(directory)))
            dll.write_bytes(b'changed synthetic transport')
            with self.assertRaisesRegex(ValueError,'identities changed'):observer.verify(directory)

    def test_frozen_native_rejections_and_success_cannot_be_changed(self):
        import copy
        rows=observer.frozen_metadata();observer.compare_metadata(rows)
        self.assertEqual([r['status'] for r in rows],[0xc0150002,0,0xc0150002])
        changed=copy.deepcopy(rows);changed[0]['status']=0
        with self.assertRaises(ValueError):observer.compare_metadata(changed)
        changed=copy.deepcopy(rows);changed[1]['records'][0]['current_context_id']=0
        with self.assertRaises(ValueError):observer.compare_metadata(changed)

    def test_remaining_layout_cases_keep_exact_owned_token_and_file_names(self):
        import observe_pe32_assembly_layout as layout
        ns={'a':'urn:schemas-microsoft-com:asm.v1'};values=layout.manifests()
        root=ET.fromstring(values['three-files'])
        self.assertEqual([n.get('name') for n in root.findall('a:file',ns)],['ctx-provider.dll','ctx-secondary.dll','ctx-unused.dll'])
        identity=ET.fromstring(values['token-exact']).find('a:assemblyIdentity',ns)
        request=ET.fromstring(observer.manifests()['token-importer']).find('a:dependency/a:dependentAssembly/a:assemblyIdentity',ns)
        self.assertEqual(identity.attrib,request.attrib)

if __name__=='__main__':unittest.main()
