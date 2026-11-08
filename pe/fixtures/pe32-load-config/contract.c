/* Original MIT legacy PE32 load-configuration / compiler GS observer. */
typedef unsigned int U32;
#define DLL __declspec(dllimport)
DLL U32 __stdcall GetLastError(void);
DLL void __stdcall SetLastError(U32);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
#ifndef CONFIG_SIZE
#define CONFIG_SIZE 72
#endif
#ifndef COOKIE_INITIAL
#define COOKIE_INITIAL 0xbb40e64eu
#endif
#ifndef ABSENT
#define ABSENT 0
#endif
#ifndef NO_COOKIE
#define NO_COOKIE 0
#endif
#ifndef FAULT
#define FAULT 0
#endif
U32 guard_before=0xa1b2c3d4u;
U32 __security_cookie=COOKIE_INITIAL;
U32 guard_after=0x88776655u;
static volatile U32 check_count;
struct LegacyConfig {U32 prefix[15];U32 *cookie;U32 seh_table,seh_count;};
_Static_assert(sizeof(struct LegacyConfig)==72,"legacy PE32 load configuration");
_Static_assert(__builtin_offsetof(struct LegacyConfig,cookie)==60,"cookie VA offset");
#if !ABSENT
const struct LegacyConfig _load_config_used={{CONFIG_SIZE},NO_COOKIE?0:&__security_cookie,0,0};
#endif
__declspec(noinline) __attribute__((no_stack_protector))
void __fastcall __security_check_cookie(U32 value) {
 check_count++;
 if(value!=__security_cookie)__builtin_trap();
}
__declspec(noinline) U32 protected_call(U32 value) {
 volatile unsigned char buffer[32];U32 i;
 for(i=0;i<32;i++)buffer[i]=(unsigned char)(value+i);
#if FAULT
 __security_cookie^=1u; /* Invalidates the real compiler-generated frame cookie. */
#endif
 return (value*33u)^(U32)buffer[value&31u]^0x12345678u;
}
#if !FAULT
static U32 words[16];
#endif
U32 entry(void) {
 U32 written=0;
#if FAULT
 const U32 marker[2]={0x46473349u,4};
 if(!WriteFile(GetStdHandle(0xfffffff5u),marker,sizeof(marker),&written,0)||written!=sizeof(marker))return 82;
 (void)protected_call(7);return 83; /* Must trap in the compiler's epilogue. */
#else
 U32 before=__security_cookie;
 words[0]=0x43323349u;words[1]=sizeof(void *);
#if !ABSENT
 {
  volatile const struct LegacyConfig *config=&_load_config_used;
  words[2]=config->prefix[0];words[6]=(config->cookie!=0);
  words[7]=(config->cookie==&__security_cookie);
 }
#endif
 words[3]=(__security_cookie!=COOKIE_INITIAL);words[4]=(__security_cookie!=0);
 words[5]=(__security_cookie!=0xbb40e64eu);
 words[8]=protected_call(7);words[9]=protected_call(11);words[10]=(check_count==2);
 words[11]=(__security_cookie==before);words[12]=guard_before;words[13]=guard_after;
 SetLastError(0x11223344u);words[14]=GetLastError();words[15]=0x99aabbccu;
 if(!WriteFile(GetStdHandle(0xfffffff5u),words,sizeof(words),&written,0)||written!=sizeof(words))return 82;
 return 0;
#endif
}
