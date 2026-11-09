float4x4 transform : register(c0);
struct VSOut { float4 position : POSITION; float4 color : COLOR0; };
VSOut main(float4 position : POSITION, float4 color : COLOR0) {
 VSOut o; o.position = mul(position, transform); o.color = color; return o;
}
