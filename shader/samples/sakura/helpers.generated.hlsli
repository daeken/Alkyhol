// Generated extraction of unchanged Microsoft function bodies; see upstream.json.
float3 HitWorldPosition()
{
    return WorldRayOrigin() + RayTCurrent() * WorldRayDirection();
}

float3 HitAttribute(float3 vertexAttribute[3], BuiltInTriangleIntersectionAttributes attr)
{
    return vertexAttribute[0] +
    attr.barycentrics.x * (vertexAttribute[1] - vertexAttribute[0]) +
    attr.barycentrics.y * (vertexAttribute[2] - vertexAttribute[0]);
}

float2 HitAttribute(float2 vertexAttribute[3], BuiltInTriangleIntersectionAttributes attr)
{
    return vertexAttribute[0] +
    attr.barycentrics.x * (vertexAttribute[1] - vertexAttribute[0]) +
    attr.barycentrics.y * (vertexAttribute[2] - vertexAttribute[0]);
}

inline void GenerateCameraRay(uint2 index, out float3 origin, out float3 direction)
{
    float2 xy = index + 0.5f; // center in the middle of the pixel.
    float2 screenPos = xy / DispatchRaysDimensions().xy * 2.0 - 1.0;

    // Invert Y for DirectX-style coordinates.
    screenPos.y = -screenPos.y;

    // Unproject the pixel coordinate into a ray.
    float4 world = mul(g_sceneCB.projectionToWorld, float4(screenPos, 0, 1));

    world.xyz /= world.w;
    origin = g_sceneCB.cameraPosition.xyz;
    direction = normalize(world.xyz - origin);
}

float3 FresnelReflectanceSchlick(in float3 I, in float3 N, in float3 f0)
{
    float cosi = saturate(dot(-I, N));
    return f0 + (1 - f0) * pow(1 - cosi, 5);
}
