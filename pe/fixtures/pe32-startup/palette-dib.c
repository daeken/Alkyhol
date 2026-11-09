/* Original MIT finite indexed-DIB witness. RGB comparisons intentionally exclude
 * the reserved high byte of 32-bit BI_RGB pixels; no alpha guarantee is inferred. */
typedef unsigned int U;typedef int L;typedef void* H;
__declspec(dllimport) H __stdcall CreateCompatibleDC(H);
__declspec(dllimport) int __stdcall GdiFlush(void);
__declspec(dllimport) H __stdcall CreateDIBSection(H,const void*,U,void**,H,U);
__declspec(dllimport) H __stdcall SelectObject(H,H);
__declspec(dllimport) int __stdcall GetObjectW(H,L,void*);
__declspec(dllimport) U __stdcall GetPixel(H,L,L);
__declspec(dllimport) H __stdcall CreateSolidBrush(U);
__declspec(dllimport) int __stdcall FillRect(H,const L*,H);
__declspec(dllimport) int __stdcall BitBlt(H,L,L,L,L,H,L,L,U);
__declspec(dllimport) int __stdcall DeleteObject(H);
__declspec(dllimport) int __stdcall DeleteDC(H);
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H,const void*,U,U*,H);
__declspec(dllimport) void __stdcall ExitProcess(U);
static U out[80],used;static void put(U n){out[used++]=n;}
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
struct Info {U header[10],colors[256];};static struct Info indexed={{40,5,(U)-2,0x80001,0,16,0,0,256,256},{0}};
U entry(void){U before=stack();put(0x38424944);put(2);for(U i=0;i<256;i++)indexed.colors[i]=i*0x10101;indexed.colors[1]=0x1d0d07;
 unsigned char *bits=0,*bottom_bits=0;U* target_bits=0;U direct[10]={40,5,(U)-2,0x200001,0,40,0,0,0,0};
 H source=CreateCompatibleDC(0),target=CreateCompatibleDC(0);put(source!=0&&target!=0);H bitmap=CreateDIBSection(source,&indexed,0,(void**)&bits,0,0);put(bitmap!=0&&bits!=0);if(!bitmap||!bits)ExitProcess(2);
 H rgb=CreateDIBSection(target,direct,0,(void**)&target_bits,0,0);put(rgb!=0&&target_bits!=0);if(!rgb||!target_bits)ExitProcess(3);
 H old_source=SelectObject(source,bitmap),old_target=SelectObject(target,rgb);put(old_source!=0&&old_target!=0);
 for(U i=0;i<16;i++)bits[i]=0x5a;for(U y=0;y<2;y++)for(U x=0;x<5;x++)bits[y*8+x]=(unsigned char)(y*5+x+1);
 U info[23];for(U i=0;i<23;i++)info[i]=0x715e7a11;
 put(GetObjectW(bitmap,84,info+1)==84&&info[0]==0x715e7a11&&info[22]==0x715e7a11&&info[1]==0&&info[2]==5&&info[3]==2&&info[4]==8&&info[5]==0x80001&&info[6]==(U)bits);
 put(GetPixel(source,0,0));put(GetPixel(source,1,1));L rectangle[4]={2,0,3,1};H brush=CreateSolidBrush(0x111111);put(brush&&FillRect(source,rectangle,brush)&&GdiFlush()&&DeleteObject(brush));put(bits[2]==17&&bits[5]==0x5a&&bits[13]==0x5a);
 for(U i=0;i<10;i++)target_bits[i]=0xaabbccdd;put(BitBlt(target,0,0,5,2,source,0,0,0xcc0020)!=0&&GdiFlush());for(U i=0;i<10;i++)put(target_bits[i]&0xffffff);
 indexed.header[2]=2;H bottom=CreateDIBSection(source,&indexed,0,(void**)&bottom_bits,0,0);put(bottom!=0&&bottom_bits!=0);if(!bottom||!bottom_bits)ExitProcess(4);
 for(U i=0;i<16;i++)bottom_bits[i]=0x5a;for(U x=0;x<5;x++){bottom_bits[x]=(unsigned char)(9+x);bottom_bits[8+x]=(unsigned char)(1+x);}
 put(SelectObject(source,bottom)==bitmap);put(GetPixel(source,1,0));put(GetPixel(source,1,1));
#ifdef QUANTIZE
 put(BitBlt(source,0,0,5,2,target,0,0,0xcc0020)!=0&&GdiFlush());
 for(U i=0;i<16;i++)put(bottom_bits[i]);
 put(bottom_bits[0]==6&&bottom_bits[8]==1&&bottom_bits[10]==17&&bottom_bits[5]==0x5a&&bottom_bits[13]==0x5a);
#endif
 put(SelectObject(source,old_source)==bottom&&SelectObject(target,old_target)==rgb);put(DeleteObject(bitmap)&&DeleteObject(bottom)&&DeleteObject(rgb));put(DeleteDC(source)&&DeleteDC(target));put(stack()==before);put(used+1);
 U done=0;int ok=WriteFile(GetStdHandle((U)-11),out,used*4,&done,0);ExitProcess(ok&&done==used*4?0:8);return 8;
}
