// Original MIT witness. No game bytes or host pointer is embedded.
#include "d3d9.h"
extern "C" __declspec(dllimport) void __stdcall ExitProcess(DWORD);
extern "C" __declspec(dllimport) void* __stdcall GetStdHandle(DWORD);
extern "C" __declspec(dllimport) BOOL __stdcall WriteFile(void*,const void*,DWORD,DWORD*,void*);
extern "C" __declspec(dllimport) HRESULT __stdcall DirectInput8Create(HINSTANCE,DWORD,const GUID*,void**,void*);
template<class R=HRESULT,class... A> static R invoke(void* p,UINT slot,A... a){using Fn=R(__stdcall*)(void*,A...);return ((Fn)(*(void***)p)[slot])(p,a...);}
static const GUID input_a={0xbf798030,0x483a,0x4da2,{0xaa,0x99,0x5d,0x64,0xed,0x36,0x97,0}};
static const GUID input_w={0xbf798031,0x483a,0x4da2,{0xaa,0x99,0x5d,0x64,0xed,0x36,0x97,0}};
static const GUID unknown={0,0,0,{0xc0,0,0,0,0,0,0,0x46}};
static const GUID keyboard={0x6f1d2b61,0xd5a0,0x11cf,{0xbf,0xc7,0x44,0x45,0x53,0x54,0,0}};
static const GUID mouse={0x6f1d2b60,0xd5a0,0x11cf,{0xbf,0xc7,0x44,0x45,0x53,0x54,0,0}};
static const GUID key={0x55728220,0xd33c,0x11cf,{0xbf,0xc7,0x44,0x45,0x53,0x54,0,0}};
static const GUID axes[3]={{0xa36d02e0,0xc9f3,0x11cf,{0xbf,0xc7,0x44,0x45,0x53,0x54,0,0}},{0xa36d02e1,0xc9f3,0x11cf,{0xbf,0xc7,0x44,0x45,0x53,0x54,0,0}},{0xa36d02e2,0xc9f3,0x11cf,{0xbf,0xc7,0x44,0x45,0x53,0x54,0,0}}};
struct ObjectFormat {const GUID* guid;DWORD offset,type,flags;};
struct Format {DWORD size,object_size,flags,data_size,count;ObjectFormat* objects;};
static DWORD out[200],used;
static void put(DWORD n){out[used++]=n;}
static DWORD stack(){DWORD n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static DWORD flush(DWORD status){DWORD done=0;WriteFile(GetStdHandle((DWORD)-11),out,used*4,&done,NULL);ExitProcess(status);return status;}
extern "C" DWORD entry(){DWORD before=stack();put(0x38494441);put(1);
 for(UINT wide=0;wide<2;wide++){
  void* factory=NULL;HRESULT hr=DirectInput8Create(GetModuleHandleW(NULL),0x800,wide?&input_w:&input_a,&factory,NULL);put(hr);if(FAILED(hr)||!factory)return flush(2);
  void* identity=NULL;HRESULT identity_hr=invoke(factory,0,&unknown,&identity);void* canonical=NULL;put(identity_hr==0&&identity&&invoke(identity,0,&unknown,&canonical)==0&&canonical==identity);if(canonical)invoke<ULONG>(canonical,2);if(identity)invoke<ULONG>(identity,2);
  void* devices[2]={};put(invoke(factory,3,&keyboard,&devices[0],(void*)NULL)==0&&devices[0]);put(invoke(factory,3,&mouse,&devices[1],(void*)NULL)==0&&devices[1]);
  if(!devices[0]||!devices[1])return flush(3);
  invoke<ULONG>(factory,2); // Devices must retain the factory without a client reference.
  for(UINT m=0;m<2;m++){
   void* device=devices[m];static DWORD caps[13];for(UINT i=0;i<13;i++)caps[i]=0x715e7a11;caps[1]=44;
   put(invoke(device,3,caps+1)==0&&caps[0]==0x715e7a11&&caps[12]==0x715e7a11&&caps[1]==44&&(caps[2]&1)&&((caps[3]&255)==(m?0x12:0x13))&&caps[5]>0);
   static DWORD info[277];for(UINT i=0;i<277;i++)info[i]=0x715e7a11;UINT n=wide?1100:580;info[1]=n;
   put(invoke(device,15,info+1)==0&&info[0]==0x715e7a11&&info[1+n/4]==0x715e7a11&&info[1]==n&&info[2]==(m?mouse.a:keyboard.a));
   static ObjectFormat objects[256];UINT count=m?11:256;
   for(UINT i=0;i<count;i++){objects[i]={m?(i<3?&axes[i]:NULL):&key,m?(i<3?i*4:12+i-3):i,m?(i<3?0x00ffff03:0x80ffff0c):(0x8000000c|(i<<8)),0};}
   Format format={24,16,2,m?20u:256u,count,objects};put(invoke(device,11,&format)==0);
   DWORD property[5]={20,16,0,0,32};put(invoke(device,6,(void*)1,property)==0);property[4]=0x715e7a11;put(invoke(device,5,(void*)1,property)==0&&property[4]==32);
   property[2]=0;property[3]=1;property[4]=0x715e7a11;
   if(!m){hr=invoke(device,5,(void*)22,property);put(hr);put(property[4]==0x715e7a11);}
   property[2]=m?0:30;
   hr=invoke(device,5,(void*)22,property);put(hr);put(property[4]); // Retain raw native default, including a possible failure.
   property[4]=0x312578ab;put(invoke(device,6,(void*)22,property)==0);property[4]=0;put(invoke(device,5,(void*)22,property)==0&&property[4]==0x312578ab);
   static DWORD object_info[146];for(UINT i=0;i<146;i++)object_info[i]=0x715e7a11;n=wide?576:316;object_info[1]=n;
   put(invoke(device,14,object_info+1,m?0u:30u,1u)==0&&object_info[0]==0x715e7a11&&object_info[1+n/4]==0x715e7a11&&object_info[1]==n&&object_info[6]==(m?0u:30u));
   static unsigned char state[256];for(UINT i=0;i<256;i++)state[i]=0x5a;hr=invoke(device,9,m?20u:256u,state);BOOL untouched=TRUE;for(UINT i=0;i<256;i++)if(state[i]!=0x5a)untouched=FALSE;put(hr==((HRESULT)0x8007000c)&&untouched);
   put(invoke(device,8)==1);invoke<ULONG>(device,2);
  }
 }
 put(stack()==before);put(used+1);return flush(0);
}
