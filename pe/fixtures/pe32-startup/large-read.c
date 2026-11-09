/* Original MIT: a real >16MiB ReadFile, complete byte digest, guards and EOF. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) U __stdcall GetModuleFileNameA(H,char*,U);
__declspec(dllimport) int __stdcall SetCurrentDirectoryA(const char*);
__declspec(dllimport) H __stdcall CreateFileA(const char*,U,U,H,U,U,H);
__declspec(dllimport) int __stdcall ReadFile(H,void*,U,U*,H);
__declspec(dllimport) int __stdcall CloseHandle(H);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
#define LENGTH (80u*1024u*1024u+3u)
static unsigned char buffer[LENGTH+64];static char directory[512];static U words[32],used;
static void put(U n){words[used++]=n;}static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
U entry(void){U before=stack();put(0x4c524746);put(1);U length=GetModuleFileNameA(0,directory,512);put(length>0&&length<512);if(!length||length>=512)ExitProcess(2);
 while(length&&directory[length-1]!='/'&&directory[length-1]!='\\')length--;if(!length)ExitProcess(3);directory[length]=0;put(SetCurrentDirectoryA(directory));
 H file=CreateFileA("owned-large-read.bin",0x80000000,7,0,3,0x80,0);put(file!=(H)-1);if(file==(H)-1)ExitProcess(4);
 for(U i=0;i<LENGTH+64;i++)buffer[i]=0x5a;
 U done=0;put(ReadFile(file,buffer+16,LENGTH+13,&done,0));put(done);
 U digest=2166136261u;for(U i=0;i<LENGTH;i++)digest=(digest^buffer[i+16])*16777619u;put(digest);
 U guards=1;for(U i=0;i<16;i++)if(buffer[i]!=0x5a)guards=0;for(U i=LENGTH+16;i<LENGTH+64;i++)if(buffer[i]!=0x5a)guards=0;put(guards);
 done=0x715e7a11;put(ReadFile(file,buffer+16,1,&done,0));put(done==0&&buffer[16]==11);put(CloseHandle(file));put(stack()==before);put(used+1);
 U transferred=0;int ok=WriteFile(GetStdHandle((U)-11),words,used*4,&transferred,0);ExitProcess(ok&&transferred==used*4?0:8);return 8;
}
