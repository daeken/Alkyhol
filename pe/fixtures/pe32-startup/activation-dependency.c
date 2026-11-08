/* Original MIT DLL: real code export and no entry point. Its nonempty manifest
 * must make the bridge's activation state unknown instead of an invented miss. */
__declspec(dllexport) unsigned int __stdcall OwnedDependency(void) { return 1; }
