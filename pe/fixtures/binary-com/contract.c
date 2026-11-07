/* Owned MIT probe: unchanged PE virtual calls through all five ID3DBlob slots.
 * Buffer contents are written before reading; no native zero-fill is assumed. */
typedef unsigned int U32;
typedef unsigned short U16;
typedef unsigned char U8;
typedef unsigned long long U64;
typedef int HRESULT;
typedef struct GUID { U32 a; U16 b,c; U8 d[8]; } GUID;
typedef struct Blob Blob;
typedef struct Vtable {
    HRESULT (__stdcall *QueryInterface)(Blob *,const GUID *,void **);
    U32 (__stdcall *AddRef)(Blob *);
    U32 (__stdcall *Release)(Blob *);
    void *(__stdcall *GetBufferPointer)(Blob *);
    U64 (__stdcall *GetBufferSize)(Blob *);
} Vtable;
struct Blob { const Vtable *vtable; };
#define DLL __declspec(dllimport)
DLL HRESULT __stdcall D3DCreateBlob(U64,Blob **);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
static const GUID unknown = {0,0,0,{0xc0,0,0,0,0,0,0,0x46}};
static const GUID blob_iid = {0x8ba5fb08,0x5195,0x40e2,{0xac,0x58,0x0d,0x98,0x9c,0x3a,1,2}};
static const GUID unsupported = {0x11223344,0,0,{0}};
static U32 words[64], used;
static U32 *volatile relocated_words = words;
#define CHECK(c,id) do { if (!(c)) ExitProcess(id); } while (0)
#define RECORD(v) do { relocated_words[used++] = (U32)(v); } while (0)
static U32 digest(volatile U8 *p,U32 size) {
    U32 i,h=0x12345678;
    for(i=0;i<size;i++) h=(h*33u)^p[i];
    return h;
}
static void fill(volatile U8 *p,U32 size) {
    U32 i;
    for(i=0;i<size;i++) p[i]=(U8)(i*13u+7u);
}
void entry(void) {
    Blob *blob=0,*base=0,*alias=0,*canonical=0,*large=0;
    void *failed=(void *)(U64)1;
    volatile U8 *p;
    U32 written=0;
    HRESULT hr;
    RECORD(0x42434f4du); /* BCOM */
    hr=D3DCreateBlob(37,&blob); RECORD(hr);
    CHECK(hr==0&&blob!=0,10);
    RECORD(blob->vtable->GetBufferSize(blob));
    CHECK(blob->vtable->GetBufferSize(blob)==37,11);
    p=(volatile U8 *)blob->vtable->GetBufferPointer(blob); CHECK(p!=0,12);
    fill(p,37); RECORD(digest(p,37));
    hr=blob->vtable->QueryInterface(blob,&unknown,(void **)&base); RECORD(hr); CHECK(hr==0&&base,13);
    hr=base->vtable->QueryInterface(base,&blob_iid,(void **)&alias); RECORD(hr); CHECK(hr==0&&alias,14);
    hr=alias->vtable->QueryInterface(alias,&unknown,(void **)&canonical); RECORD(hr); CHECK(hr==0&&canonical,15);
    RECORD(canonical==base);
    RECORD(alias->vtable->GetBufferPointer(alias)==(void *)p);
    RECORD(alias->vtable->GetBufferSize(alias));
    RECORD(blob->vtable->QueryInterface(blob,&unsupported,&failed));
    RECORD(failed==0);
    RECORD(blob->vtable->AddRef(blob));
    RECORD(canonical->vtable->Release(canonical));
    RECORD(alias->vtable->Release(alias));
    RECORD(base->vtable->Release(base));
    RECORD(blob->vtable->Release(blob));
    RECORD(digest(p,37)); /* Original owned reference must still keep data alive. */
    RECORD(blob->vtable->Release(blob));
    hr=D3DCreateBlob(16384,&large); RECORD(hr); CHECK(hr==0&&large,16);
    RECORD(large->vtable->GetBufferSize(large));
    CHECK(large->vtable->GetBufferSize(large)==16384,17);
    p=(volatile U8 *)large->vtable->GetBufferPointer(large); CHECK(p!=0,18);
    fill(p,16384); RECORD(digest(p,16384));
    RECORD(large->vtable->Release(large));
    CHECK(WriteFile(GetStdHandle(0xfffffff5u),words,used*4,&written,0)&&written==used*4,19);
    ExitProcess(0);
}
