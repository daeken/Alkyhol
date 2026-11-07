"""Host build-intake assertions; compiler execution has separate physical evidence."""
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
from tools import fetch_cpp_deps
from tools.build_original_miniengine import project_items,shader_command

class ProjectIntakeTests(unittest.TestCase):
    def test_windows_paths_metadata_and_ignored_conditional_overrides(self):
        with tempfile.TemporaryDirectory() as directory:
            project=Path(directory)/'sample.vcxproj'
            project.write_text('<Project xmlns="http://schemas.microsoft.com/developer/msbuild/2003"><ItemGroup><ClCompile Include="Math\\Vector.cpp"/><FxCompile Include="Shaders\\Rays.hlsl"><ShaderType>Library</ShaderType><ShaderModel>6.3</ShaderModel><EntryPointName> </EntryPointName><AdditionalOptions>-HV 2017 -Zpr</AdditionalOptions><ShaderModel Condition="debug">6.9</ShaderModel></FxCompile></ItemGroup></Project>')
            cpp=project_items(project,'ClCompile');self.assertEqual(cpp,[(Path(directory)/'Math/Vector.cpp',{})])
            shader,meta=project_items(project,'FxCompile')[0]
            self.assertEqual(shader,Path(directory)/'Shaders/Rays.hlsl');self.assertEqual(meta['ShaderModel'],'6.3')
            self.assertEqual(meta['EntryPointName'],'')

    def test_library_profiles_preserve_row_major_and_have_no_entry_flag(self):
        cmd=shader_command(Path('Rays.hlsl'),{'ShaderType':'Library','ShaderModel':'6.3','AdditionalOptions':'-HV 2017 -Zpr'},Path('Rays.h'),Path('dxc'))
        self.assertIn('lib_6_3',cmd);self.assertIn('-Zpr',cmd);self.assertIn('2017',cmd)
        self.assertNotIn('-E',cmd);self.assertEqual(cmd[-1],'Rays.hlsl');self.assertIn('g_pRays',cmd)

    def test_desktop_defaults_and_explicit_vertex_entry(self):
        cmd=shader_command(Path('Tone.hlsl'),{},Path('Tone.h'),Path('dxc'))
        self.assertIn('cs_6_2',cmd);self.assertIn('2021',cmd);self.assertIn('main',cmd)
        cmd=shader_command(Path('Triangle.hlsl'),{'ShaderType':'Vertex','EntryPointName':'VSMain'},Path('Triangle.h'),Path('dxc'))
        self.assertIn('vs_6_2',cmd);self.assertIn('VSMain',cmd)
        with self.assertRaises(KeyError):shader_command(Path('x.hlsl'),{'ShaderType':'unknown'},Path('x.h'),Path('dxc'))

class DependencyIntakeTests(unittest.TestCase):
    def test_wrong_revision_or_tracked_edit_fails_without_resetting_source(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);repo=root/'local/upstream/test';repo.mkdir(parents=True)
            marker=repo/'source.cpp';marker.write_text('keep this')
            manifest=root/'manifest.json';manifest.write_text(json.dumps({'repositories':{'test':{'url':'unused','revision':'pinned'}}}))
            with patch.object(fetch_cpp_deps,'ROOT',root),patch.object(fetch_cpp_deps,'MANIFEST',manifest):
                with patch.object(fetch_cpp_deps.subprocess,'check_output',return_value='other\n'),self.assertRaises(ValueError):fetch_cpp_deps.ensure('test',clone=False)
                with patch.object(fetch_cpp_deps.subprocess,'check_output',return_value='pinned\n'),patch.object(fetch_cpp_deps.subprocess,'run',return_value=SimpleNamespace(returncode=1)),self.assertRaises(ValueError):fetch_cpp_deps.ensure('test',clone=False)
                self.assertEqual(marker.read_text(),'keep this')
                with self.assertRaises(ValueError):
                    import shutil
                    shutil.rmtree(repo)
                    fetch_cpp_deps.ensure('test',clone=False)
