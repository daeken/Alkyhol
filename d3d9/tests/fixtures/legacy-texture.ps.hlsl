sampler2D image : register(s0);
float4 main(float4 color : COLOR0) : COLOR0 { return tex2D(image,color.xy); }
