// Generated extraction of unchanged Microsoft function bodies; see upstream.json.
[shader("closesthit")]
void FloorClosestHitShader(inout RayPayload payload, in MyAttributes attr)
{
    float3 hitPosition = HitWorldPosition();
    uint baseIndex = PrimitiveIndex() * 3;
    uint offset = baseIndex * 4;
    uint3 indices = IndicesCube.Load3(offset);

    float3 albedo = g_cubeCB.albedo.rgb;
    float4 sampled = float4(1.0, 1.0, 1.0, 1.0);
    float3 triangleNormal;

    // Procedural UVs
    float2 baseUV = frac(hitPosition.xz * 1.0);
    sampled.rgb = TrunkTexture.SampleLevel(TrunkSampler, baseUV, 0).rgb;
    
    float3 vertexNormals[3] =
    {
        VerticesCube[indices.x].normal,
        VerticesCube[indices.y].normal,
        VerticesCube[indices.z].normal
    };
        
    triangleNormal = HitAttribute(vertexNormals, attr);
    float3 baseColor = albedo * sampled.rgb;
    float3 finalColor;

    // Trace reflection ray if surface is dark
    if (all(sampled.rgb < 0.1) && payload.recursionDepth < MAX_RECURSION_DEPTH)
    {
        Ray reflectionRay;
        reflectionRay.Origin = hitPosition + triangleNormal * 0.001f;
        reflectionRay.Direction = reflect(WorldRayDirection(), triangleNormal);

        float4 reflectionColor = TraceRadianceRay(reflectionRay, payload.recursionDepth);
        float3 fresnel = FresnelReflectanceSchlick(WorldRayDirection(), triangleNormal, baseColor);
        float3 refColor = lerp(baseColor, reflectionColor.rgb, fresnel) * 1.7f;
        finalColor = refColor;
    }
    else
    {
        finalColor = baseColor * 1.0f;
    }
    payload.color = float4(finalColor, g_cubeCB.albedo.w);
}

[shader("closesthit")]
void TrunkClosestHitShader(inout RayPayload payload, in MyAttributes attr)
{
    float3 hitPosition = HitWorldPosition();
    uint baseIndex = PrimitiveIndex() * 3;
    uint offset = baseIndex * 4;
    uint3 indices = IndicesLeaves.Load3(offset);

    // Albedo is defined per shape or material
    float3 albedo = g_cubeCB.albedo.rgb;
    float4 sampled = float4(1.0, 1.0, 1.0, 1.0);
    float3 triangleNormal;
        
    // Retrieve corresponding vertex normals for the triangle vertices.
    float3 vertexNormals[3];
    vertexNormals[0] = VerticesTrunk[indices.x].normal;
    vertexNormals[1] = VerticesTrunk[indices.y].normal;
    vertexNormals[2] = VerticesTrunk[indices.z].normal;
    triangleNormal = HitAttribute(vertexNormals, attr);
            
    // Sample texture from 3 planes (triplanar mapping)
    float2 uvX = hitPosition.yz * 0.5;
    float2 uvY = hitPosition.xz * 0.5;
    float2 uvZ = hitPosition.xy * 0.5;
    
    float3 texX = TrunkTexture.SampleLevel(TrunkSampler, uvX, 0).rgb;
    float3 texY = TrunkTexture.SampleLevel(TrunkSampler, uvY, 0).rgb;
    float3 texZ = TrunkTexture.SampleLevel(TrunkSampler, uvZ, 0).rgb;
    
    // Blend based on normal
    float3 blendWeights = abs(triangleNormal);
    blendWeights = normalize(max(blendWeights, 0.00001));
    sampled.rgb = texX * blendWeights.x + texY * blendWeights.y + texZ * blendWeights.z;

    float3 finalColor = albedo * sampled.rgb * 0.5f;
    payload.color = float4(finalColor, g_cubeCB.albedo.w);
}

[shader("closesthit")]
void LeavesClosestHitShader(inout RayPayload payload, in MyAttributes attr)
{
    float3 hitPosition = HitWorldPosition();
    uint baseIndex = PrimitiveIndex() * 3;
    uint offset = baseIndex * 4;
    uint3 indices = IndicesLeaves.Load3(offset);

    float3 albedo = g_cubeCB.albedo.rgb;
    float3 pastelPurple = float3(0.8, 0.7, 0.9); 
    float3 pastelPink = float3(1.0, 0.8, 0.9);
    float3 darkPink = float3(0.85, 0.4, 0.6); 

    // Simple hash to pick one of the three colors for sakura petals.
    uint hash = PrimitiveIndex() % 3;
    float3 sampled = (hash == 0) ? pastelPurple : (hash == 1) ? pastelPink :darkPink;
    float3 triangleNormal;
    float3 vertexNormals[3];
        
    vertexNormals[0] = VerticesLeaves[indices.x].normal;
    vertexNormals[1] = VerticesLeaves[indices.y].normal;
    vertexNormals[2] = VerticesLeaves[indices.z].normal;
   
    triangleNormal = HitAttribute(vertexNormals, attr);

    float3 finalColor;
    float3 lightDir = normalize(g_sceneCB.lightPosition.xyz - hitPosition);
    float NdotL = saturate(dot(triangleNormal, lightDir));

    finalColor = albedo * sampled.rgb * g_sceneCB.lightDiffuseColor.rgb * NdotL;
    payload.color = float4(finalColor, g_cubeCB.albedo.w);
}

[shader("closesthit")]
void BushClosestHitShader(inout RayPayload payload, in MyAttributes attr)
    {
    float3 hitPosition = HitWorldPosition();
    uint baseIndex = PrimitiveIndex() * 3;
    uint offset = baseIndex * 4;
    uint3 indices = IndicesBush.Load3(offset);

    // Albedo is defined per shape or material
    float3 albedo = g_cubeCB.albedo.rgb;
    float4 sampled = float4(1.0, 1.0, 1.0, 1.0);
    float3 triangleNormal;

    // Retrieve corresponding vertex normals for the triangle vertices.
    float3 vertexNormals[3];
        
    vertexNormals[0] = VerticesBush[indices.x].normal;
    vertexNormals[1] = VerticesBush[indices.y].normal;
    vertexNormals[2] = VerticesBush[indices.z].normal;
        
    triangleNormal = HitAttribute(vertexNormals, attr);

    float3 vertexTexCoords[3];
        
    vertexTexCoords[0] = VerticesBush[indices.x].uv;
    vertexTexCoords[1] = VerticesBush[indices.y].uv;
    vertexTexCoords[2] = VerticesBush[indices.z].uv;
        
    float2 interpolatedTexCoord = HitAttribute(vertexTexCoords, attr).xy;

    // Sample the texture with the modified coordinates
    sampled.rgb = BushTexture.SampleLevel(BushSampler, interpolatedTexCoord, 0).rgb;
        
    float3 baseColor = albedo * sampled.rgb;
    float3 lightDir = normalize(g_sceneCB.lightPosition.xyz - hitPosition);
    float NdotL = saturate(dot(triangleNormal, lightDir));
    float3 finalColor = baseColor * g_sceneCB.lightDiffuseColor.rgb * NdotL;
    payload.color = float4(finalColor, g_cubeCB.albedo.w);
}

[shader("closesthit")]
void TCubeClosestHitShader(inout RayPayload payload, in MyAttributes attr)
{
    float3 hitPosition = HitWorldPosition();
    uint baseIndex = PrimitiveIndex() * 3;
    uint offset = baseIndex * 4;
    uint3 indices = IndicesCube.Load3(offset);

    // Albedo is defined per shape or material
    float3 albedo = g_cubeCB.albedo.rgb;
    float4 sampled = float4(1.0, 1.0, 1.0, 1.0);
    float3 triangleNormal;

    // Retrieve corresponding vertex normals for the triangle vertices.
    float3 vertexNormals[3];
        
    vertexNormals[0] = VerticesTrunk[indices.x].normal;
    vertexNormals[1] = VerticesTrunk[indices.y].normal;
    vertexNormals[2] = VerticesTrunk[indices.z].normal;
        
    triangleNormal = HitAttribute(vertexNormals, attr);
        
    // Trace a reflection ray
    Ray reflectionRay = { hitPosition + triangleNormal * 0.5f, reflect(WorldRayDirection(), triangleNormal) };
    float4 reflectionColor = TraceRadianceRay(reflectionRay, payload.recursionDepth);
    float3 fresnelR = FresnelReflectanceSchlick(WorldRayDirection(), triangleNormal, g_cubeCB.albedo.xyz);
    float4 reflectedColor = 0.5 * float4(fresnelR, 1) * reflectionColor;
    payload.color = reflectedColor;
}
