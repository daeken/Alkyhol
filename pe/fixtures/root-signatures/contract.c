/* Owned MIT probe. One unchanged PE calls the real D3D12 serializers and then
 * their Win64 blob vtables. Layout cases mirror our frozen SDK source probe.
 * No device, compiler DLL, SDK headers, CRT, or GPU is required on the Mac. */
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
typedef struct Range { U32 kind,count,reg,space,offset; } Range;
typedef struct Table { U32 count; const Range *ranges; } Table;
typedef struct Parameter {
    U32 kind;
    union { Table table; U32 constants[3]; U32 descriptor[2]; } payload;
    U32 visibility;
} Parameter;
typedef struct Description {
    U32 count; const Parameter *parameters;
    U32 sampler_count; const U32 (*samplers)[13]; U32 flags;
} Description;
/* Actual versioned union is larger for newer versions. Only v1.0 is accessed. */
typedef struct Versioned { U32 version; Description description; } Versioned;
_Static_assert(sizeof(Parameter)==32,"Windows parameter layout");
_Static_assert(sizeof(Description)==40,"Windows root description layout");
_Static_assert(__builtin_offsetof(Versioned,description)==8,"Windows union alignment");
#define DLL __declspec(dllimport)
DLL HRESULT __stdcall D3D12SerializeRootSignature(const Description *,U32,Blob **,Blob **);
DLL HRESULT __stdcall D3D12SerializeVersionedRootSignature(const Versioned *,Blob **,Blob **);
DLL void *__stdcall GetStdHandle(U32);
DLL int __stdcall WriteFile(void *,const void *,U32,U32 *,void *);
DLL __declspec(noreturn) void __stdcall ExitProcess(U32);
static const GUID unknown = {0,0,0,{0xc0,0,0,0,0,0,0,0x46}};
static U8 output[4096];
static U8 *volatile relocated_output=output;
static U32 used;
#define CHECK(c,id) do { if (!(c)) ExitProcess(id); } while (0)
static void record(U32 value) {
    U32 i; CHECK(used+4<=sizeof(output),20);
    for(i=0;i<4;i++) relocated_output[used++]=(U8)(value>>(i*8));
}
static void capture(const Description *desc) {
    Blob *blob=0,*other=0,*error=(Blob *)(U64)1,*identity=0;
    Versioned versioned={1,*desc};
    U64 size,i;
    const U8 *bytes,*other_bytes;
    HRESULT legacy=D3D12SerializeRootSignature(desc,1,&blob,&error);
    CHECK(legacy==0&&blob&&error==0,21); record((U32)legacy);
    error=(Blob *)(U64)1;
    HRESULT modern=D3D12SerializeVersionedRootSignature(&versioned,&other,&error);
    CHECK(modern==0&&other&&error==0,22); record((U32)modern);
    size=blob->vtable->GetBufferSize(blob); CHECK(size>0&&size<1024,23); record((U32)size);
    bytes=(const U8 *)blob->vtable->GetBufferPointer(blob);
    other_bytes=(const U8 *)other->vtable->GetBufferPointer(other);
    CHECK(bytes&&other_bytes&&other->vtable->GetBufferSize(other)==size,24);
    for(i=0;i<size;i++) CHECK(bytes[i]==other_bytes[i],25);
    record(1);
    CHECK(blob->vtable->QueryInterface(blob,&unknown,(void **)&identity)==0&&identity,26);
    record(identity==blob);
    record(identity->vtable->Release(identity));
    CHECK(used+size<=sizeof(output),27);
    for(i=0;i<size;i++) relocated_output[used++]=bytes[i];
    record(other->vtable->Release(other));
    record(blob->vtable->Release(blob));
}
void entry(void) {
    Description desc={0};
    Parameter constants={0},descriptors[7]={0},tables[2]={0},mixed[3]={0};
    Range ranges[3]={{0,3,0,0,0xffffffffu},{2,2,1,0,8},{1,2,4,0,0xffffffffu}};
    Range sampler_range={3,4,4,0,0xffffffffu};
    U32 samplers[2][13]={{0}},i,written=0;
    Blob *blob=(Blob *)(U64)1,*error=0;
    U64 size;
    const U8 *text;
    record(0x52545045u); record(6);
    capture(&desc);
    constants.kind=1; constants.payload.constants[0]=3;
    constants.payload.constants[1]=2; constants.payload.constants[2]=64;
    desc.count=1; desc.parameters=&constants; capture(&desc);
    for(i=0;i<7;i++) {
        descriptors[i].kind=2+i%3; descriptors[i].payload.descriptor[0]=i;
        descriptors[i].payload.descriptor[1]=2;
    }
    desc.count=7; desc.parameters=descriptors; capture(&desc);
    tables[0].payload.table.count=3; tables[0].payload.table.ranges=ranges;
    tables[1].payload.table.count=1; tables[1].payload.table.ranges=&sampler_range;
    tables[1].visibility=5;
    desc.count=2; desc.parameters=tables; capture(&desc);
    for(i=0;i<2;i++) {
        samplers[i][0]=i?0x95:0x15; /* Comparison/ordinary MIN_MAG_MIP_LINEAR. */
        samplers[i][1]=1; samplers[i][2]=3; samplers[i][3]=4;
        samplers[i][5]=16; samplers[i][6]=4; samplers[i][7]=2;
        samplers[i][9]=0x7f7fffff; samplers[i][10]=i; samplers[i][12]=5;
    }
    desc.sampler_count=2; desc.samplers=samplers; capture(&desc);
    mixed[0].kind=1; mixed[0].payload.constants[1]=2;
    mixed[0].payload.constants[2]=8; mixed[0].visibility=1;
    mixed[1].kind=2; mixed[1].payload.descriptor[1]=2; mixed[1].visibility=5;
    mixed[2].kind=3; mixed[2].payload.descriptor[0]=3; mixed[2].payload.descriptor[1]=6;
    desc.count=3; desc.parameters=mixed; desc.sampler_count=0; desc.samplers=0; desc.flags=1;
    capture(&desc);
    /* Valid pointers, invalid 65-DWORD budget. Diagnostic spelling is backend-specific. */
    constants.payload.constants[2]=65; desc.count=1; desc.parameters=&constants; desc.flags=0;
    record((U32)D3D12SerializeRootSignature(&desc,1,&blob,&error));
    record(blob==0); CHECK(error!=0,28); record(1);
    size=error->vtable->GetBufferSize(error); CHECK(size>1&&size<65536,29);
    text=(const U8 *)error->vtable->GetBufferPointer(error); CHECK(text!=0,30);
    record(text[size-1]==0); record(error->vtable->Release(error));
    CHECK(WriteFile(GetStdHandle(0xfffffff5u),output,used,&written,0)&&written==used,31);
    ExitProcess(0);
}
