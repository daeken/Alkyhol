/* Alkyhol MIT: owned PCM, guarded COM calls and real asynchronous playback. */
typedef unsigned int U;typedef unsigned short W;typedef void *H;
#define API __declspec(dllimport)
API U __stdcall DirectSoundCreate8(const void *,void **,void *);
API U __stdcall DirectSoundEnumerateA(int (__stdcall *)(const void *,const char *,const char *,void *),void *);
API H __stdcall GetDesktopWindow(void);API H __stdcall CreateEventW(void *,int,int,const W *);
API U __stdcall WaitForSingleObject(H,U);API int __stdcall CloseHandle(H);API void __stdcall Sleep(U);
API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
API H __stdcall LoadLibraryA(const char *);API H __stdcall GetProcAddress(H,const char *);API int __stdcall FreeLibrary(H);
typedef U (__stdcall *F0)(void *);typedef U (__stdcall *F1)(void *,U);typedef U (__stdcall *F2)(void *,U,U);typedef U (__stdcall *F3)(void *,U,U,U);typedef U (__stdcall *F4)(void *,U,U,U,U);typedef U (__stdcall *F7)(void *,U,U,U,U,U,U,U);
#define V(o,n) (*(void ***)(o))[n]
#define C0(o,n) ((F0)V(o,n))(o)
#define C1(o,n,a) ((F1)V(o,n))(o,(U)(a))
#define C2(o,n,a,b) ((F2)V(o,n))(o,(U)(a),(U)(b))
#define C3(o,n,a,b,c) ((F3)V(o,n))(o,(U)(a),(U)(b),(U)(c))
#define C4(o,n,a,b,c,d) ((F4)V(o,n))(o,(U)(a),(U)(b),(U)(c),(U)(d))
#define C7(o,n,a,b,c,d,e,f,g) ((F7)V(o,n))(o,(U)(a),(U)(b),(U)(c),(U)(d),(U)(e),(U)(f),(U)(g))
static U words[128],used;static void put(U n){words[used++]=n;}static U enumerated;
static int __stdcall enumerate(const void *guid,const char *name,const char *module,void *context){(void)guid;(void)module;enumerated+=(context==(void *)0x1234&&name&&*name);return 0;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static const unsigned char iid8[16]={0x49,0xa4,0x25,0x68,0x24,0x75,0x82,0x4d,0x92,0xf,0x50,0xe3,0x6a,0xb3,0xab,0x1e};
static const unsigned char notify_iid[16]={0x83,7,0x21,0xb0,0xcd,0x89,0xd0,0x11,0xaf,8,0,0xa0,0xc9,0x25,0xcd,0x16};
static const unsigned char unknown[16]={0,0,0,0,0,0,0,0,0xc0,0,0,0,0,0,0,0x46};
static const unsigned char bad[16]={1,2,3};
#pragma pack(push,1)
struct Format {W tag,channels;U rate,bytes;W align,bits,extra;};
#pragma pack(pop)
struct Desc {U size,flags,bytes,reserved;struct Format *format;unsigned char algorithm[16];};
U entry(void){
 U before=stack();put(0x44534e32);put(1);
 H module=LoadLibraryA("dsound.dll");put(module!=0);put(GetProcAddress(module,"DirectSoundCreate8")!=0);put(FreeLibrary(module)!=0);
 put(DirectSoundEnumerateA(enumerate,(void *)0x1234)==0);put(enumerated==1);
 void *device=0;put(DirectSoundCreate8(0,&device,0)==0&&device!=0);
 U caps[25]={96};caps[24]=0xfeedface;put(C1(device,4,caps)==0);put(caps[0]==96&&caps[24]==0xfeedface);put((caps[1]&0x20)!=0&&(caps[1]&0x40)==0);
 put(C2(device,6,GetDesktopWindow(),2)==0);
 struct Desc primary={36,1,0,0,0,{0}};void *p=0;put(C3(device,3,&primary,&p,0)==0&&p!=0);
 struct Format format={1,2,48000,192000,4,16,0};put(C1(p,14,&format)==0);put(C0(p,2)==0);
 struct Desc desc={36,0x1e8,8192,0,&format,{0}};void *buffer=0;put(C3(device,3,&desc,&buffer,0)==0&&buffer!=0);
 void *as8=0;put(C2(buffer,0,iid8,&as8)==0&&as8==buffer);put(C0(as8,2)>0);
 void *absent=(void *)1;put(C2(buffer,0,bad,&absent)==0x80004002&&absent==0);
 U bcaps[6]={20};bcaps[5]=0xfeedbeef;put(C1(buffer,3,bcaps)==0);put(bcaps[2]==8192&&bcaps[5]==0xfeedbeef);
 U count=0;put(C3(buffer,5,0,0,&count)==0&&count==18);struct Format observed;put(C3(buffer,5,&observed,18,&count)==0&&observed.rate==48000&&observed.channels==2&&observed.bits==16);
 put(C1(buffer,15,(U)-2000)==0);U value=0;put(C1(buffer,6,&value)==0&&value==(U)-2000);put(C1(buffer,15,1)==0x80070057);put(C1(buffer,6,&value)==0&&value==(U)-2000);
 put(C1(buffer,16,(U)-1000)==0);put(C1(buffer,7,&value)==0&&value==(U)-1000);put(C1(buffer,17,44100)==0);put(C1(buffer,8,&value)==0&&value==44100);put(C1(buffer,17,0)==0);
 void *first=0,*second=0;U n1=0,n2=0;
 put(C7(buffer,11,8188,8,&first,&n1,&second,&n2,0)==0);put(first&&second&&n1==4&&n2==4);*(U *)first=0;*(U *)second=0;
 put(C4(buffer,19,(U)first+4,n1,second,n2)==0x80070057);put(C4(buffer,19,first,n1,second,n2)==0);
 put(C7(buffer,11,0,0,&first,&n1,&second,&n2,2)==0);put(n1==8192&&n2==0&&second==0);
 for(U n=0;n<8192/2;++n)((short *)first)[n]=(n/32)%2?500:-500;
 put(C4(buffer,19,first,n1,second,n2)==0);put(C1(buffer,13,0)==0);
 void *notify=0;put(C2(buffer,0,notify_iid,&notify)==0&&notify!=0);void *identity=0;put(C2(notify,0,unknown,&identity)==0&&identity==buffer);put(C0(identity,2)>0);
 H event=CreateEventW(0,1,0,0);put(event!=0);U notification[2]={4096,(U)event};put(C2(notify,3,1,notification)==0);
 put(C3(buffer,12,0,0,1)==0);put(WaitForSingleObject(event,2000)==0);
 U play=0,write=0;put(C2(buffer,4,&play,&write)==0);put(play<8192&&write<8192&&(play%4)==0&&(write%4)==0);
 U state=0;put(C1(buffer,9,&state)==0&&(state&5)==5);put(C0(buffer,18)==0);put(C1(buffer,9,&state)==0&&(state&1)==0);
 put(C2(buffer,4,&play,&write)==0);Sleep(40);U stopped=play;put(C2(buffer,4,&play,&write)==0&&play==stopped);
 put(C0(buffer,20)==0);put(C0(notify,2)>0);put(C0(buffer,2)==0);put(C0(device,2)==0);put(CloseHandle(event)!=0);put(stack()==before);put(used+1);
 U done=0;if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
