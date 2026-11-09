/* Original MIT GDI-to-native-window drawing witness on primary and worker threads. */
typedef unsigned int U;typedef int L;typedef unsigned short W;typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetModuleHandleW(const W *);API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
API H __stdcall CreateThread(H,U,U(__stdcall *)(void *),void *,U,U *);API U __stdcall WaitForSingleObject(H,U);API int __stdcall GetExitCodeThread(H,U *);API int __stdcall CloseHandle(H);
API W __stdcall RegisterClassExW(const void *);API H __stdcall CreateWindowExW(U,const W *,const W *,U,L,L,L,L,H,H,H,void *);API L __stdcall DefWindowProcW(H,U,U,L);API int __stdcall ShowWindow(H,int);API int __stdcall UpdateWindow(H);API int __stdcall DestroyWindow(H);API int __stdcall UnregisterClassA(const char *,H);
API H __stdcall CreateDIBSection(H,const void *,U,void **,H,U);API int __stdcall GetObjectW(H,int,void *);
API H __stdcall LoadBitmapW(H,const W *);API H __stdcall GetDC(H);API int __stdcall ReleaseDC(H,H);API H __stdcall CreateCompatibleDC(H);API H __stdcall SelectObject(H,H);API int __stdcall DeleteObject(H);API int __stdcall DeleteDC(H);API int __stdcall BitBlt(H,L,L,L,L,H,L,L,U);API U __stdcall GetPixel(H,L,L);
static U out[32],used;static void put(U v){out[used++]=v;}static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static L __stdcall procedure(H h,U msg,U w,L l){return DefWindowProcW(h,msg,w,l);}
static U __stdcall draw(void *parameter){(void)parameter;U before=stack();put(0x47444932);put(4);H instance=GetModuleHandleW(0);U wc[]={48,3,(U)procedure,0,0,(U)instance,0,0,0,0,(U)L"AlkyholGdiWindow",0};put(RegisterClassExW(wc)!=0);H window=CreateWindowExW(0,L"AlkyholGdiWindow",L"Alkyhol owned GDI pixels",0xcf0000,100,100,240,160,0,0,instance,0);put(window!=0);ShowWindow(window,1);UpdateWindow(window);H bitmap=LoadBitmapW(instance,(W *)101),dc=GetDC(window),memory=CreateCompatibleDC(dc);put(bitmap!=0&&dc!=0&&memory!=0);U header[]={40,3,(U)-2,0x200001,0,0,0,0,0,0};U *bits=0;H dib=CreateDIBSection(dc,header,0,(void **)&bits,0,0);put(dib!=0&&bits!=0);U metadata[6];put(GetObjectW(dib,24,metadata)==24&&metadata[5]==(U)bits);H previous=SelectObject(memory,dib);put(previous!=0);H source=CreateCompatibleDC(dc),previous_source=SelectObject(source,bitmap);if(!BitBlt(memory,0,0,3,2,source,0,0,0xcc0020))__builtin_trap();put((bits[0]&0xffffff)==0x0c0b0a);bits[0]=0x00112233;SelectObject(source,previous_source);DeleteDC(source);put(BitBlt(dc,3,5,3,2,memory,0,0,0xcc0020)!=0);put(GetPixel(dc,3,5));put(GetPixel(dc,3,6));put(SelectObject(memory,previous)==dib);put(ReleaseDC(window,dc)!=0);put(DeleteObject(bitmap)!=0&&DeleteObject(dib)!=0);put(DeleteDC(memory)!=0);put(DestroyWindow(window)!=0);put(UnregisterClassA("AlkyholGdiWindow",instance)!=0);put(stack()==before);put(used+1);U done=0;if(!WriteFile(GetStdHandle((U)-11),out,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;}
U entry(void){
#if MODE == 0
 return draw(0);
#else
 H thread=CreateThread(0,0,draw,0,0,0);if(!thread||WaitForSingleObject(thread,(U)-1)!=0)__builtin_trap();U code;if(!GetExitCodeThread(thread,&code)||code||!CloseHandle(thread))__builtin_trap();return 0;
#endif
}
