/* Original MIT microtests for NLS calls actually reached during CRT startup. */
typedef unsigned int U32;
typedef unsigned short U16;
__declspec(dllimport) void *__stdcall GetStdHandle(U32);
__declspec(dllimport) int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
__declspec(dllimport) int __stdcall GetStringTypeW(U32,const U16 *,int,U16 *);
__declspec(dllimport) int __stdcall MultiByteToWideChar(U32,U32,const char *,int,U16 *,int);
__declspec(dllimport) int __stdcall LCMapStringW(U32,U32,const U16 *,int,U16 *,int);
__declspec(dllimport) U32 __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(U32);
static void emit(const void *data,U32 bytes) {
    U32 written=0;
    if(!WriteFile(GetStdHandle((U32)-11),data,bytes,&written,0)||written!=bytes)__builtin_trap();
}
U32 entry(void) {
#if MODE==3
    /* Native-only capture for the next actual original CRT call. No host pass. */
    char bytes[256];U16 wide[256],lower[512],upper[512];
    for(U32 i=0;i<256;++i)bytes[i]=(char)i;
    if(MultiByteToWideChar(1252,0,bytes,256,wide,256)!=256)__builtin_trap();
    SetLastError(0);int lo=LCMapStringW(0,0x100,wide,256,lower,512);U32 le=GetLastError();
    SetLastError(0);int up=LCMapStringW(0,0x200,wide,256,upper,512);U32 ue=GetLastError();
    if(lo<0||lo>512||up<0||up>512)__builtin_trap();
    U32 header[5]={0x4c434d31,(U32)lo,(U32)up,le,ue};emit(header,sizeof(header));
    emit(lower,(U32)lo*2);emit(upper,(U32)up*2);
    U32 detail[2]={0x4c434d32,256};emit(detail,sizeof(detail));
    for(U32 i=0;i<256;++i){
        U16 one_lower[8],one_upper[8];
        SetLastError(0);lo=LCMapStringW(0,0x100,wide+i,1,one_lower,8);le=GetLastError();
        SetLastError(0);up=LCMapStringW(0,0x200,wide+i,1,one_upper,8);ue=GetLastError();
        if(lo<0||lo>8||up<0||up>8)__builtin_trap();
        U32 row[5]={wide[i],(U32)lo,(U32)up,le,ue};emit(row,sizeof(row));
        emit(one_lower,(U32)lo*2);emit(one_upper,(U32)up*2);
    }
#elif MODE==0
    const U16 source[5]={'a','A',0,9,'a'},terminated[3]={'A',0,'z'};
    U16 output[6]={0,0,0,0,0,0xbeef},tail[3]={0,0,0xbeef};
    U32 ok=(U32)GetStringTypeW(1,source,5,output);
    U32 end=(U32)GetStringTypeW(1,terminated,-1,tail);
    U32 words[12]={0x4e4c5332,ok,output[0],output[1],output[2],output[3],output[4],output[5],end,tail[0],tail[1],tail[2]};
    emit(words,sizeof(words));
#else
    U32 boot=0x53543332;emit(&boot,4);
    const U16 source[3]={'A',0x2603,'a'};U16 output[3]={0,0,0};
    (void)GetStringTypeW(MODE==1?1:2,source,3,output);
    emit(&boot,4); /* No post-call output for unsupported units/info levels. */
#endif
    return 0;
}
