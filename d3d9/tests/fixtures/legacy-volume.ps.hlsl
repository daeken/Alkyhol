// Original MIT volume sampler for DAO resource validation; no game shader code.
sampler3D image : register(s0);
float4 main(float4 coordinate : COLOR0) : COLOR0
{
    return tex3D(image, coordinate.xyz);
}
