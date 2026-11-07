// Copyright (c) Alkyhol contributors. MIT licensed.
// Nonzero registers and a separate space keep helper indexing independent of
// D3D register numbers. All lengths must come from the bound buffer extents.
ByteAddressBuffer bytes : register(t3, space1);
struct Element { float3 position; uint tag; float2 uv; };
StructuredBuffer<Element> vertices : register(t7);
RWStructuredBuffer<uint> output : register(u5);
RWByteAddressBuffer writable : register(u9);
[numthreads(1,1,1)]
void CSMain() {
    uint a,b,c,d,e,f;
    bytes.GetDimensions(a);
    vertices.GetDimensions(b,c);
    output.GetDimensions(d,e);
    writable.GetDimensions(f);
    output[0]=a; output[1]=b; output[2]=c;
    output[3]=d; output[4]=e; output[5]=f;
    output[6]=vertices[b-1].tag; output[7]=bytes.Load(a-4);
    writable.Store(0,0x13579bdf);
}
