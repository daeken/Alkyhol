// Copyright (c) Alkyhol contributors. MIT licensed.
// Host scaling bridge for already encoded diagnostic/capture colours.
Texture2D<float4> image : register(t0);
SamplerState linearClamp : register(s0);
float4 main(float4 position:SV_Position,float2 uv:TexCoord0):SV_Target0 {return image.SampleLevel(linearClamp,uv,0);}
