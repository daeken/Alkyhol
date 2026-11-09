/* Original MIT finite PE32 witness for the original D3DX allocation/memory path.
 * No game bytes. Calls real MSVCRT imports with cdecl pointer/size arguments. */
typedef unsigned int U;typedef void* H;
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
__declspec(dllimport) H __cdecl malloc(U);
__declspec(dllimport) void __cdecl free(H);
__declspec(dllimport) H __cdecl scalar_new(U) __asm("??2@YAPAXI@Z");
__declspec(dllimport) void __cdecl scalar_delete(H) __asm("??3@YAXPAX@Z");
__declspec(dllimport) H __cdecl array_new(U) __asm("??_U@YAPAXI@Z");
__declspec(dllimport) void __cdecl array_delete(H) __asm("??_V@YAXPAX@Z");
__declspec(dllimport) H __cdecl memcpy(H,const void*,U);
__declspec(dllimport) H __cdecl memmove(H,const void*,U);
__declspec(dllimport) H __cdecl memset(H,int,U);
__declspec(dllimport) char* __cdecl strchr(const char*,int);
__declspec(dllimport) int __cdecl strncmp(const char*,const char*,U);
static U words[160],used;
static void put(U v){words[used++]=v;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static H (__cdecl *allocators[])(U)={scalar_new,array_new,malloc};
static void (__cdecl *releasers[])(H)={scalar_delete,array_delete,free};
static const U sizes[]={0,1,24,4097,65539};
U entry(void){U initial=stack();put(0x4d545243);put(1);
 for(U kind=0;kind<3;kind++){
  unsigned char* blocks[5];
  for(U i=0;i<5;i++){U before=stack();blocks[i]=allocators[kind](sizes[i]);put(blocks[i]!=0);put(((U)blocks[i]&7)==0);put(stack()==before);if(!blocks[i])ExitProcess(2);for(U j=0;j<sizes[i];j++)blocks[i][j]=(unsigned char)(j*17+i*23+kind*7);}
  for(U i=0;i<5;i++){U valid=1,sum=0;for(U j=0;j<sizes[i];j++){if(blocks[i][j]!=(unsigned char)(j*17+i*23+kind*7))valid=0;sum+=blocks[i][j];}put(valid);put(sum);U before=stack();releasers[kind](blocks[i]);put(stack()==before);}
  U before=stack();releasers[kind](0);put(stack()==before);
 }
 unsigned char source[32],dest[32];for(U i=0;i<32;i++){source[i]=(unsigned char)(i*7+3);dest[i]=0x5a;}
 put(memcpy(dest+3,source+5,17)==dest+3);U valid=1;for(U i=0;i<32;i++)if(dest[i]!=(i>=3&&i<20?source[i+2]:0x5a))valid=0;put(valid);
 for(U i=0;i<32;i++)dest[i]=(unsigned char)i;
 put(memmove(dest+3,dest,20)==dest+3);valid=1;for(U i=0;i<32;i++)if(dest[i]!=(i>=3&&i<23?i-3:i))valid=0;put(valid);
 put(memmove(dest,dest+3,20)==dest);valid=1;for(U i=0;i<20;i++)if(dest[i]!=i)valid=0;put(valid);
 put(memset(dest+4,0x1234,15)==dest+4);valid=1;for(U i=4;i<19;i++)if(dest[i]!=0x34)valid=0;put(valid);
 put(memset(dest+4,0,0)==dest+4&&dest[4]==0x34);
 const char* a[]={"a","b","abc","abc","\xff","\x01","abc","a","z"};
 const char* b[]={"b","a","abc","abd","\x01","\xff","xyz","ab","a"};
 const U counts[]={1,1,8,2,1,1,0,4,3};
 for(U i=0;i<9;i++)put((U)strncmp(a[i],b[i],counts[i]));
 const char text[]="abca";const int needles[]={'a','c',0,'d',0x161};
 for(U i=0;i<5;i++){char* p=strchr(text,needles[i]);put(p?(U)(p-text):0xffffffff);}
 const char high[]="\xff\x01";char* p=strchr(high,255);put(p?(U)(p-high):0xffffffff);
 put(stack()==initial);put(used+1);U done=0;int ok=WriteFile(GetStdHandle((U)-11),words,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
