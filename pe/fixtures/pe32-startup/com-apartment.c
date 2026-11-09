/* Original MIT COM initialization witness, driven by DAO's original FMOD path.
 * Successful initialization balances, changed modes and actual worker isolation. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) U __stdcall CoInitialize(H);
__declspec(dllimport) U __stdcall CoInitializeEx(H,U);
__declspec(dllimport) void __stdcall CoUninitialize(void);
__declspec(dllimport) U __stdcall CoCreateInstance(const void*,H,U,const void*,H*);
static const U clsid[4]={0xad38f099,0x43ea587e,0xe09ec0a6,0xc9b64a29};
static const U iid[4]={0,0,0xc0,0x46000000};
__declspec(dllimport) H __stdcall CreateThread(H,U,U(__stdcall*)(H),H,U,U*);
__declspec(dllimport) U __stdcall WaitForSingleObject(H,U);
__declspec(dllimport) int __stdcall GetExitCodeThread(H,U*);
__declspec(dllimport) int __stdcall CloseHandle(H);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
static U words[64],used;static volatile U worker_words[5];
static void put(U n){words[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static U __stdcall worker(H parameter){(void)parameter;U before=stack();
 worker_words[0]=CoInitializeEx(0,0);worker_words[1]=CoInitializeEx(0,4);worker_words[2]=CoInitialize(0);
 CoUninitialize();CoUninitialize();worker_words[3]=CoInitialize(0);CoUninitialize();worker_words[4]=stack()==before;return 0x3344;
}
U entry(void){U before=stack();put(0x50414f43);put(1);
 H guarded[3]={(H)0x715e7a11,(H)1,(H)0x715e7a11};
 put(CoCreateInstance(clsid,0,7,iid,guarded+1));put(guarded[0]==(H)0x715e7a11&&guarded[1]==0&&guarded[2]==(H)0x715e7a11);
 put(CoInitialize(0));put(CoInitialize(0));put(CoInitializeEx(0,0));
 guarded[1]=(H)1;put(CoCreateInstance(clsid,0,7,iid,guarded+1));put(guarded[0]==(H)0x715e7a11&&guarded[1]==0&&guarded[2]==(H)0x715e7a11);
 U id=0;H thread=CreateThread(0,0,worker,0,0,&id);put(thread!=0&&id!=0);if(!thread)ExitProcess(2);
 put(WaitForSingleObject(thread,5000));U exit=0;put(GetExitCodeThread(thread,&exit)&&exit==0x3344);for(U i=0;i<5;i++)put(worker_words[i]);
 CoUninitialize();put(CoInitializeEx(0,0));CoUninitialize();
 put(CoInitializeEx(0,0));put(CoInitializeEx(0,8));put(CoInitialize(0));CoUninitialize();CoUninitialize();
 put(CoInitialize(0));CoUninitialize();put(CloseHandle(thread));put(stack()==before);put(used+1);
 U done=0;int ok=WriteFile(GetStdHandle((U)-11),words,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
