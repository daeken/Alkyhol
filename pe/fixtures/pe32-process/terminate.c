/* Original MIT DLL with a genuine non-returning imported ExitProcess call. */
typedef unsigned int U32;
__declspec(dllimport) void __stdcall ExitProcess(U32);
__declspec(dllexport) void __stdcall Finish(U32 status) {
    ExitProcess(status);
    __builtin_trap();
}
