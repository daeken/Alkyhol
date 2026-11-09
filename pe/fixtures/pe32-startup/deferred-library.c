/* Original MIT DLL witness: original machine-code ATTACH/THREAD/DETACH state. */
typedef unsigned int U;static U attached,threads,*sink;
int __stdcall DllMain(void *h,U reason,void *reserved){(void)h;(void)reserved;if(reason==1){++attached;return !REJECT;}if(reason==2)++threads;if(reason==0&&sink)*sink=0xde000000|threads;return 1;}
__declspec(dllexport) U __cdecl AttachCount(void){return attached;}
__declspec(dllexport) U __cdecl ThreadCount(void){return threads;}
__declspec(dllexport) void __cdecl SetSink(U *p){sink=p;*p=0xac000000|attached;}
