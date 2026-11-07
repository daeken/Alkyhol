// Generated extraction of unchanged Microsoft function bodies; see upstream.json.
float4 TraceRadianceRay(in Ray ray, in int currentRayRecursionDepth)
{
    // Stop tracing if maximum depth is reached.
    if (currentRayRecursionDepth >= MAX_RECURSION_DEPTH)
    {
        return float4(0, 0, 0, 0);
    }

    // Set the ray's extents.
    RayDesc rayDesc;
    rayDesc.Origin = ray.Origin;
    rayDesc.Direction = ray.Direction;
    // Set TMin to a non-zero small value to avoid aliasing issues due to floating - point errors.
    rayDesc.TMin = 0.001;
    rayDesc.TMax = 10000;
    RayPayload rayPayload = { float4(0, 0, 0, 0), currentRayRecursionDepth + 1 };
    TraceRay(Scene, 0, ~0, 0, 1, 0, rayDesc, rayPayload);

    return rayPayload.color;
}
