/* Original MIT family probe; all files and object state are owned fixtures. */
typedef unsigned int U;typedef int L;typedef unsigned short W;typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
API U __stdcall GetLastError(void);API void __stdcall SetLastError(U);API U __stdcall GetModuleFileNameA(H,char *,U);
API H __stdcall GetCurrentThread(void);API L __stdcall InterlockedExchangeAdd(volatile L *,L);
API H __stdcall CreateEventA(H,int,int,const char *);API H __stdcall CreateSemaphoreA(H,L,L,const char *);
API int __stdcall ReleaseSemaphore(H,L,L *);API U __stdcall WaitForSingleObject(H,U);API int __stdcall CloseHandle(H);
API int __stdcall SetEvent(H);API int __stdcall SetCurrentDirectoryA(const char *);API U __stdcall GetCurrentDirectoryA(U,char *);
API int __stdcall CreateDirectoryA(const char *,H);API U __stdcall GetFileAttributesA(const char *);API U __stdcall GetFileAttributesW(const W *);
API H __stdcall CreateFileA(const char *,U,U,H,U,U,H);API U __stdcall GetFileSize(H,U *);API int __stdcall GetFileSizeEx(H,void *);
API int __stdcall WritePrivateProfileStringW(const W *,const W *,const W *,const W *);
API U __stdcall GetPrivateProfileStringW(const W *,const W *,const W *,W *,U,const W *);
static W ini_output[64];
static U words[160],used;static char root[512],path[512],cwd[512];static W wide[512];static U length;static volatile L atomic;
static void put(U n){words[used++]=n;}static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static void name(const char *leaf){U i;for(i=0;i<length;++i)path[i]=root[i];U j=0;do{path[i+j]=leaf[j];}while(leaf[j++]);for(i=0;path[i];++i)wide[i]=(W)(unsigned char)path[i];wide[i]=0;}
U entry(void){U n=GetModuleFileNameA(0,root,512);if(!n||n>=448)__builtin_trap();while(n&&root[n-1]!='\\')--n;if(!n)__builtin_trap();const char tail[]="root";for(U i=0;i<sizeof(tail);++i)root[n+i]=tail[i];length=n+4;put(0x46414d32);put(4);U before=stack();(void)before;
#if MODE == 1
 H event=CreateEventA(0,0,0,0);(void)ReleaseSemaphore(event,1,0);__builtin_trap();
#elif MODE == 2
 if(!SetCurrentDirectoryA(root))__builtin_trap();name("\\file");H h=CreateFileA(path,0x80000000,7,0,3,128,0);(void)GetFileSizeEx(h,(void *)1);__builtin_trap();
#elif MODE == 3
 (void)InterlockedExchangeAdd((L *)((char *)&atomic+1),1);__builtin_trap();
#elif MODE == 4
 (void)GetFileAttributesA("file");__builtin_trap();
#elif MODE == 5
 (void)SetCurrentDirectoryA("non-ascii-\xe9");__builtin_trap();
#elif MODE == 6
 if(!SetCurrentDirectoryA(root))__builtin_trap();(void)GetCurrentDirectoryA(512,(char *)1);__builtin_trap();
#elif MODE == 7
 (void)CreateEventA(0,0,0,"named");__builtin_trap();
#elif MODE == 8
 H sem=CreateSemaphoreA(0,0,3,0);(void)ReleaseSemaphore(sem,1,(L *)1);__builtin_trap();
#else
 put(GetCurrentThread()==(H)-2);atomic=0x7fffffff;put((U)InterlockedExchangeAdd(&atomic,1));put((U)atomic);put((U)InterlockedExchangeAdd(&atomic,-1));put((U)atomic);atomic=-1;put((U)InterlockedExchangeAdd(&atomic,2));put((U)atomic);
 H event=CreateEventA(0,0,1,0);put(event!=0);put(WaitForSingleObject(event,0));put(WaitForSingleObject(event,0));put(SetEvent(event)!=0);put(WaitForSingleObject(event,0));put(CloseHandle(event)!=0);
 H sem=CreateSemaphoreA(0,0,3,0);put(sem!=0);L previous=0x71717171;put(ReleaseSemaphore(sem,2,&previous)!=0);put((U)previous);put(WaitForSingleObject(sem,0));put(WaitForSingleObject(sem,0));put(WaitForSingleObject(sem,0));put(ReleaseSemaphore(sem,3,0)!=0);
 previous=0x71717171;put(ReleaseSemaphore(sem,1,&previous)!=0);put(GetLastError());put((U)previous);put(ReleaseSemaphore(sem,0,&previous)!=0);put(GetLastError());put((U)previous);put(ReleaseSemaphore(sem,-1,&previous)!=0);put(GetLastError());put((U)previous);put(CloseHandle(sem)!=0);
 put(SetCurrentDirectoryA(root)!=0);put(GetCurrentDirectoryA(0,0)==length+1);put(GetCurrentDirectoryA(512,cwd)==length);U equal=1;for(U i=0;i<=length;++i)if(cwd[i]!=root[i])equal=0;put(equal);
 name("\\new");put(CreateDirectoryA(path,0)!=0);put(CreateDirectoryA(path,0)!=0);put(GetLastError());put(GetFileAttributesA(path));put(GetFileAttributesW(wide));
 name("\\file");put(GetFileAttributesA(path));put(GetFileAttributesW(wide));H file=CreateFileA(path,0x80000000,7,0,3,128,0);put(file!=(H)-1);U high=0x71717171;put(GetFileSize(file,&high));put(high);put(GetFileSize(file,0));
 struct {U before,low,high,after;} size={0x715e7a11,0x71717171,0x71717171,0x715e7a11};put(GetFileSizeEx(file,&size.low)!=0);put(size.low);put(size.high);put(size.before);put(size.after);put(CloseHandle(file)!=0);
 name("\\absent");put(GetFileAttributesA(path));put(GetLastError());put(GetFileAttributesW(wide));put(GetLastError());name("\\missing\\child");put(GetFileAttributesA(path));put(GetLastError());
 name("\\query.ini");put(WritePrivateProfileStringW(L"S",L"K",L"value",wide)!=0);SetLastError(0xface1234);put(GetPrivateProfileStringW(L"S",L"Missing",L"fallback",ini_output,64,wide));put(GetLastError());U equal_ini=1;const W fallback[]=L"fallback";for(U i=0;i<sizeof(fallback)/sizeof(W);++i)if(ini_output[i]!=fallback[i])equal_ini=0;put(equal_ini);SetLastError(0xface1234);put(GetPrivateProfileStringW(L"Other",L"K",0,ini_output,64,wide));put(GetLastError());put(ini_output[0]==0);
 put(stack()==before);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
#endif
}
