// Generated extraction of unchanged Microsoft function bodies; see upstream.json.
[shader("miss")]
void MyMissShader(inout RayPayload payload)
{
    float4 background = float4(0.05f, 0.02f, 0.08f, 1.0f);
    
    // Create sky position and ray direction
    float3 rayOrigin = WorldRayOrigin();
    float3 rayDir = normalize(WorldRayDirection());
    float3 skyPosition = rayDir * 10.0f;
    float fractalDensity = CalculateStarNest(skyPosition, rayDir);

    // Create soft purple light glow
    float3 blue = float3(0.4, 0.6, 1.0); // Soft blue
    float3 pink = float3(1.0, 0.6, 0.8); // Soft pink
    float3 glow = lerp(blue, pink, fractalDensity);

    float3 L = normalize(g_sceneCB.lightPosition.xyz - skyPosition);
    float lightD = saturate(dot(rayDir, L)); 
    glow *= lerp(0.8 + 0.2 * lightD, 4.0, fractalDensity);
    
    float3 finalColor = lerp(background.rgb, glow, fractalDensity * 1.3);
    payload.color = float4(finalColor, 1.0f);
}
