// Generated extraction of unchanged Microsoft function bodies; see upstream.json.
[shader("closesthit")]
void Hit(inout RayPayload payload, in BuiltInTriangleIntersectionAttributes attr)
{
    payload.RayHitT = RayTCurrent();
    if (payload.SkipShading)
    {
        return;
    }

    uint materialID = MaterialID;
    uint triangleID = PrimitiveIndex();

    RayTraceMeshInfo info = g_meshInfo[materialID];

    const uint3 ii = Load3x16BitIndices(info.m_indexOffsetBytes + PrimitiveIndex() * 3 * 2);
    const float2 uv0 = GetUVAttribute(info.m_uvAttributeOffsetBytes + ii.x * info.m_attributeStrideBytes);
    const float2 uv1 = GetUVAttribute(info.m_uvAttributeOffsetBytes + ii.y * info.m_attributeStrideBytes);
    const float2 uv2 = GetUVAttribute(info.m_uvAttributeOffsetBytes + ii.z * info.m_attributeStrideBytes);

    float3 bary = float3(1.0 - attr.barycentrics.x - attr.barycentrics.y, attr.barycentrics.x, attr.barycentrics.y);
    float2 uv = bary.x * uv0 + bary.y * uv1 + bary.z * uv2;

    const float3 normal0 = asfloat(g_attributes.Load3(info.m_normalAttributeOffsetBytes + ii.x * info.m_attributeStrideBytes));
    const float3 normal1 = asfloat(g_attributes.Load3(info.m_normalAttributeOffsetBytes + ii.y * info.m_attributeStrideBytes));
    const float3 normal2 = asfloat(g_attributes.Load3(info.m_normalAttributeOffsetBytes + ii.z * info.m_attributeStrideBytes));
    float3 vsNormal = normalize(normal0 * bary.x + normal1 * bary.y + normal2 * bary.z);
    
    const float3 tangent0 = asfloat(g_attributes.Load3(info.m_tangentAttributeOffsetBytes + ii.x * info.m_attributeStrideBytes));
    const float3 tangent1 = asfloat(g_attributes.Load3(info.m_tangentAttributeOffsetBytes + ii.y * info.m_attributeStrideBytes));
    const float3 tangent2 = asfloat(g_attributes.Load3(info.m_tangentAttributeOffsetBytes + ii.z * info.m_attributeStrideBytes));
    float3 vsTangent = normalize(tangent0 * bary.x + tangent1 * bary.y + tangent2 * bary.z);

    // Reintroduced the bitangent because we aren't storing the handedness of the tangent frame anywhere.  Assuming the space
    // is right-handed causes normal maps to invert for some surfaces.  The Sponza mesh has all three axes of the tangent frame.
    //float3 vsBitangent = normalize(cross(vsNormal, vsTangent)) * (isRightHanded ? 1.0 : -1.0);
    const float3 bitangent0 = asfloat(g_attributes.Load3(info.m_bitangentAttributeOffsetBytes + ii.x * info.m_attributeStrideBytes));
    const float3 bitangent1 = asfloat(g_attributes.Load3(info.m_bitangentAttributeOffsetBytes + ii.y * info.m_attributeStrideBytes));
    const float3 bitangent2 = asfloat(g_attributes.Load3(info.m_bitangentAttributeOffsetBytes + ii.z * info.m_attributeStrideBytes));
    float3 vsBitangent = normalize(bitangent0 * bary.x + bitangent1 * bary.y + bitangent2 * bary.z);

    // TODO: Should just store uv partial derivatives in here rather than loading position and caculating it per pixel
    const float3 p0 = asfloat(g_attributes.Load3(info.m_positionAttributeOffsetBytes + ii.x * info.m_attributeStrideBytes));
    const float3 p1 = asfloat(g_attributes.Load3(info.m_positionAttributeOffsetBytes + ii.y * info.m_attributeStrideBytes));
    const float3 p2 = asfloat(g_attributes.Load3(info.m_positionAttributeOffsetBytes + ii.z * info.m_attributeStrideBytes));

    float3 worldPosition = WorldRayOrigin() + WorldRayDirection() * RayTCurrent();

    //---------------------------------------------------------------------------------------------
    // Compute partial derivatives of UV coordinates:
    //
    //  1) Construct a plane from the hit triangle
    //  2) Intersect two helper rays with the plane:  one to the right and one down
    //  3) Compute barycentric coordinates of the two hit points
    //  4) Reconstruct the UV coordinates at the hit points
    //  5) Take the difference in UV coordinates as the partial derivatives X and Y

    // Normal for plane
    float3 triangleNormal = normalize(cross(p2 - p0, p1 - p0));

    // Helper rays
    uint2 threadID = DispatchRaysIndex().xy;
    float3 ddxOrigin, ddxDir, ddyOrigin, ddyDir;
    GenerateCameraRay(uint2(threadID.x + 1, threadID.y), ddxOrigin, ddxDir);
    GenerateCameraRay(uint2(threadID.x, threadID.y + 1), ddyOrigin, ddyDir);

    // Intersect helper rays
    float3 xOffsetPoint = RayPlaneIntersection(worldPosition, triangleNormal, ddxOrigin, ddxDir);
    float3 yOffsetPoint = RayPlaneIntersection(worldPosition, triangleNormal, ddyOrigin, ddyDir);

    // Compute barycentrics 
    float3 baryX = BarycentricCoordinates(xOffsetPoint, p0, p1, p2);
    float3 baryY = BarycentricCoordinates(yOffsetPoint, p0, p1, p2);

    // Compute UVs and take the difference
    float3x2 uvMat = float3x2(uv0, uv1, uv2);
    float2 ddxUV = mul(baryX, uvMat) - uv;
    float2 ddyUV = mul(baryY, uvMat) - uv;

    //---------------------------------------------------------------------------------------------

    const float3 diffuseColor = g_localTexture.SampleGrad(g_s0, uv, ddxUV, ddyUV).rgb;
    float3 normal;
    float3 specularAlbedo = float3(0.56, 0.56, 0.56);
    float specularMask = 0;     // TODO: read the texture
    float gloss = 128.0;
    {
        normal = g_localNormal.SampleGrad(g_s0, uv, ddxUV, ddyUV).rgb * 2.0 - 1.0;
        AntiAliasSpecular(normal, gloss);
        float3x3 tbn = float3x3(vsTangent, vsBitangent, vsNormal);
        normal = normalize(mul(normal, tbn));
    }
    
    float3 outputColor = AmbientColor * diffuseColor * texSSAO[DispatchRaysIndex().xy];

    float shadow = 1.0;
    if (UseShadowRays)
    {
        float3 shadowDirection = SunDirection;
        float3 shadowOrigin = worldPosition;
        RayDesc rayDesc = { shadowOrigin,
            0.1f,
            shadowDirection,
            FLT_MAX };
        RayPayload shadowPayload;
        shadowPayload.SkipShading = true;
        shadowPayload.RayHitT = FLT_MAX;
        TraceRay(g_accel, RAY_FLAG_ACCEPT_FIRST_HIT_AND_END_SEARCH,~0,0,1,0,rayDesc,shadowPayload);
        if (shadowPayload.RayHitT < FLT_MAX)
        {
            shadow = 0.0;
        }
    }
    else
    {
        // TODO: This could be pre-calculated once per vertex if this mul per pixel was a concern
        float4 shadowCoord = mul(ModelToShadow, float4(worldPosition, 1.0f));
        shadow = GetShadow(shadowCoord.xyz);
    }
    
    const float3 viewDir = normalize(-WorldRayDirection());

    outputColor +=  shadow * ApplyLightCommon(
        diffuseColor,
        specularAlbedo,
        specularMask,
        gloss,
        normal,
        viewDir,
        SunDirection,
        SunColor);

    // TODO: Should be passed in via material info
    if (IsReflection)
    {
        float reflectivity = normals[DispatchRaysIndex().xy].w;
        outputColor = g_screenOutput[DispatchRaysIndex().xy].rgb + reflectivity * outputColor;
    }

    g_screenOutput[DispatchRaysIndex().xy] = float4(outputColor, 1.0);
}
