/* Original MIT: DAO's FMOD probes a NULL module after absent dsound.dll.
 * Test actual EXE name/ordinal lookup, not a fabricated missing-DLL handle. */
typedef unsigned int U;typedef void* H;typedef U(__cdecl *F)(U);
__declspec(dllimport) H __stdcall GetModuleHandleA(const char*);
__declspec(dllimport) H __stdcall GetProcAddress(H,const char*);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
__declspec(dllexport) U __cdecl OwnedExport(U n){return n^0x39acd142;}
static U out[32],used;static void put(U n){out[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){U before=stack();put(0x4c554e47);put(1);H module=GetModuleHandleA(0);put(module!=0);
 F direct=(F)GetProcAddress(module,"OwnedExport"),implicit=(F)GetProcAddress(0,"OwnedExport");put(direct==OwnedExport);put(implicit==OwnedExport);if(!direct||!implicit)ExitProcess(2);
 put(direct(17)==(17^0x39acd142));put(implicit(29)==(29^0x39acd142));
 SetLastError(0xface1234);put(GetProcAddress(0,"OwnedAbsentExport")==0);put(GetLastError());
 SetLastError(0xface1234);put(GetProcAddress(module,"OwnedAbsentExport")==0);put(GetLastError());
 put((F)GetProcAddress(0,(const char*)1)==OwnedExport);
 SetLastError(0xface1234);put(GetProcAddress(0,(const char*)9999)==0);put(GetLastError());
 put(stack()==before);put(used+1);U done=0;int ok=WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
