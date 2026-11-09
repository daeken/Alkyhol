/* Original MIT thread/TEB/TLS/LastError/suspend/join ABI witness. */
typedef unsigned int U;typedef int L;typedef void *H;
#define API __declspec(dllimport)
API H __stdcall CreateThread(H,U,U(__stdcall *)(void *),void *,U,U *);API U __stdcall ResumeThread(H);API U __stdcall GetCurrentThreadId(void);API U __stdcall GetThreadId(H);API int __stdcall GetExitCodeThread(H,U *);API U __stdcall WaitForSingleObject(H,U);API int __stdcall CloseHandle(H);API void __stdcall ExitThread(U);API void __stdcall Sleep(U);
API U __stdcall TlsAlloc(void);API int __stdcall TlsFree(U);API void *__stdcall TlsGetValue(U);API int __stdcall TlsSetValue(U,void *);API void __stdcall SetLastError(U);API U __stdcall GetLastError(void);
API U __stdcall FlsAlloc(void(__stdcall *)(void *));API int __stdcall FlsFree(U);API void *__stdcall FlsGetValue(U);API int __stdcall FlsSetValue(U,void *);API L __stdcall InterlockedExchangeAdd(volatile L *,L);
API int __stdcall InitializeCriticalSectionAndSpinCount(void *,U);API void __stdcall EnterCriticalSection(void *);API int __stdcall TryEnterCriticalSection(void *);API void __stdcall LeaveCriticalSection(void *);API void __stdcall DeleteCriticalSection(void *);
API H __stdcall CreateEventW(void *,int,int,const unsigned short *);API int __stdcall SetEvent(H);
API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
U _tls_index;
__asm__(".globl __tls_array\n.set __tls_array,0x2c");
#pragma section(".tls",read,write)
__declspec(allocate(".tls")) char tls_start=0;
static U shared_template_target=0x715e7a11;
__declspec(thread) __declspec(align(16)) U static_seed=0x12345678;
__declspec(thread) U static_zero;
__declspec(thread) U *static_pointer=&shared_template_target;
#pragma section(".tls$ZZZ",read,write)
__declspec(allocate(".tls$ZZZ")) char tls_end=0;
typedef void (__stdcall *Callback)(void *,U,void *);
static Callback const callbacks[1]={0};
struct TlsDirectory {void *start,*end;U *index;Callback const *callbacks;U zero,flags;};
const struct TlsDirectory _tls_used={&tls_start,&tls_end,&_tls_index,callbacks,0,0x00500000};
static U fls,section[6];static H attempted;static volatile L destroyed;
static void __stdcall destroy(void *value){(void)InterlockedExchangeAdd(&destroyed,(L)(U)value);}
static U index,parent_teb,parent_id,valid[2],ids[2],tebs[2],words[32],used;
static void put(U v){words[used++]=v;}static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}static U teb(void){U n;__asm__ volatile("movl %%fs:0x18,%0":"=r"(n));return n;}
static U __stdcall worker(void *parameter){U n=(U)parameter,i=n-1,before=stack();U value=teb(),id=GetCurrentThreadId();U good_static=(static_seed==0x12345678&&static_zero==0&&static_pointer==&shared_template_target&&*static_pointer==0x715e7a11);static_seed=n;static_zero=0x200+n;ids[i]=id;tebs[i]=value;valid[i]=(value!=parent_teb&&id!=parent_id&&id!=0&&TlsGetValue(index)==0);U tid,base,limit;__asm__ volatile("movl %%fs:0x24,%0;movl %%fs:4,%1;movl %%fs:8,%2":"=r"(tid),"=r"(base),"=r"(limit));valid[i]&=(tid==id&&before>=limit&&before<base);valid[i]&=TlsSetValue(index,(void *)(0x100+n))!=0;valid[i]&=FlsGetValue(fls)==0;valid[i]&=FlsSetValue(fls,(void *)n)!=0;
 if(n==1){valid[i]&=TryEnterCriticalSection(section)==0;SetEvent(attempted);}EnterCriticalSection(section);LeaveCriticalSection(section);SetLastError(0x1000+n);Sleep(0);valid[i]&=GetLastError()==0x1000+n;valid[i]&=TlsGetValue(index)==(void *)(0x100+n);valid[i]&=FlsGetValue(fls)==(void *)n;valid[i]&=good_static&&static_seed==n&&static_zero==0x200+n;valid[i]&=stack()==before;if(n==1)ExitThread(0x101);return 0x102;}
U entry(void){U before=stack();parent_teb=teb();parent_id=GetCurrentThreadId();static_seed=0xaaaaaaaa;static_zero=0xbbbbbbbb;index=TlsAlloc();if(index==(U)-1)__builtin_trap();put(0x54485232);put(4);fls=FlsAlloc(destroy);if(fls==(U)-1||!FlsSetValue(fls,(void *)4))__builtin_trap();if(!InitializeCriticalSectionAndSpinCount(section,10))__builtin_trap();EnterCriticalSection(section);attempted=CreateEventW(0,1,0,0);if(!attempted)__builtin_trap();put(TlsSetValue(index,(void *)0xaaaa)!=0);U one=0,two=0,status=0;H a=CreateThread(0,0,worker,(void *)1,4,&one);put(a!=0);put(GetThreadId(a)==one&&one!=parent_id);put(GetExitCodeThread(a,&status)!=0&&status==259);put(ResumeThread(a)==1);if(WaitForSingleObject(attempted,(U)-1)!=0)__builtin_trap();H b=CreateThread(0,0,worker,(void *)2,0,&two);put(b!=0);put(one!=two);LeaveCriticalSection(section);put(WaitForSingleObject(a,(U)-1)==0);put(WaitForSingleObject(b,(U)-1)==0);put(GetExitCodeThread(a,&status)!=0&&status==0x101);put(GetExitCodeThread(b,&status)!=0&&status==0x102);put(valid[0]);put(valid[1]);put(ids[0]==one&&ids[1]==two&&tebs[0]!=tebs[1]);put(TlsGetValue(index)==(void *)0xaaaa&&static_seed==0xaaaaaaaa&&static_zero==0xbbbbbbbb);put(CloseHandle(a)!=0);put(CloseHandle(b)!=0);put(TlsFree(index)!=0);put(destroyed==3);put(FlsGetValue(fls)==(void *)4);put(FlsFree(fls)!=0);put(destroyed==7);DeleteCriticalSection(section);CloseHandle(attempted);put(stack()==before);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;}
