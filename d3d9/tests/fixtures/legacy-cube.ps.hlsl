// Original MIT shader for DAO's cube texture sampling contract; no game code.
samplerCUBE image : register(s0);
float4 main(float4 direction : COLOR0) : COLOR0
{
    return texCUBE(image, direction.xyz);
}
