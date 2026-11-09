/* Original MIT test: real first-pass chained x86 SEH continuation. */
typedef unsigned int U;typedef void *H;
#define API __declspec(dllimport)
API void __stdcall RaiseException(U,U,U,const U *);API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
#ifndef MODE
#define MODE 0
#endif
static U words[12],used,inner,outer,valid=1,params[]={0x12345678,0xfedcba98};
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static U __cdecl search(U *record,U *frame,U *context,U dispatcher){(void)frame;(void)dispatcher;++inner;if(record[0]!=0x715e7a11||record[1]!=0||record[4]!=2||record[5]!=params[0]||record[6]!=params[1]||(context[0]&0x10007)!=0x10007||!context[46]||!record[3]||!context[49])valid=0;return 1;}
static U __cdecl handle(U *record,U *frame,U *context,U dispatcher){(void)record;(void)frame;(void)dispatcher;++outer;
#if MODE == 3
 context[44]=0x1234;
#else
 (void)context;
#endif
 return 0;}
U entry(void){U previous;__asm__ volatile("movl %%fs:0,%0":"=r"(previous));U a[]={previous,(U)handle},b[]={(U)a,(U)search};__asm__ volatile("movl %0,%%fs:0"::"r"(b):"memory");U before=stack();
#if MODE == 1
 __asm__ volatile("movl %0,%%fs:0"::"r"((U)-1):"memory");
#endif
 RaiseException(0x715e7a11,MODE==2,2,params);__asm__ volatile("movl %0,%%fs:0"::"r"(previous):"memory");words[used++]=0x53454832;words[used++]=4;words[used++]=inner;words[used++]=outer;words[used++]=valid;words[used++]=(stack()==before);words[used++]=7;U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;}
