/* Owned MIT witness for the actually reached legacy ANSI registry call. */
typedef unsigned int U;typedef void *H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void *,U,U *,H);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall RegOpenKeyA(H,const char*,H*);
static U words[32],used;
static void put(U n){words[used++]=n;}
static U stack(void){U s;__asm__ volatile("movl %%esp,%0":"=r"(s));return s;}
U entry(void){U before=stack();put(0x4c474552);put(1);
 H out[3]={(H)0x715e7a11,(H)0xf00df00d,(H)0x715e7a11};
 SetLastError(0xface1234);put(RegOpenKeyA((H)0x80000001,"Software\\AlkyholOwnedMissingLegacy\\{5B40D859-43AF-4528-9A38-6EA2F41759E8}",out+1));
 put(out[1]==0&&out[0]==(H)0x715e7a11&&out[2]==(H)0x715e7a11);put(GetLastError());
 SetLastError(0xface1234);put(RegOpenKeyA((H)0x80000001,0,out+1));put(out[1]==(H)0x80000001&&out[0]==(H)0x715e7a11&&out[2]==(H)0x715e7a11);put(GetLastError());
 put(RegOpenKeyA((H)0x80000002,"",out+1));put(out[1]==(H)0x80000002&&out[0]==(H)0x715e7a11&&out[2]==(H)0x715e7a11);put(GetLastError());
 put(stack()==before);put(used+1);U done=0;return WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)&&done==used*4?0:7;
}
