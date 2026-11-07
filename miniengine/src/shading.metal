// Adapted from Microsoft MiniEngine DiffuseHitShaderLib.hlsl / ToneMappingUtility.hlsli.
// Copyright (c) Microsoft. MIT; see licenses/ and upstream.json.
struct Vertex {packed_float3 position;packed_float2 uv;packed_float3 normal,tangent,bitangent;};
struct MeshInfo {uint base,material;};
struct Params {float4 eye,right,up,forward;uint width,height,mode,padding;};
struct MaterialTextures {
 array<texture2d<float>,27> diffuse [[id(0)]];
 array<texture2d<float>,27> normal [[id(27)]];
 array<texture2d<float>,27> specular [[id(54)]];
};
constexpr sampler wrap_linear(coord::normalized,address::repeat,filter::linear,mip_filter::linear,max_anisotropy(8));
float3 camera_direction(float2 pixel,constant Params& p){
 float2 screen=(pixel+.5f)/float2(p.width,p.height)*2-1;screen.y=-screen.y;
 return normalize(p.forward.xyz+p.right.xyz*screen.x*(float(p.width)/p.height)*tan(M_PI_F/8.0f)+p.up.xyz*screen.y*tan(M_PI_F/8.0f));
}
struct Surface {float3 position,normal,diffuse;float reflectivity,specular,gloss;uint repaired;};
float3 plane_point(float3 point,float3 normal,float3 origin,float3 direction){return origin+direction*(dot(normal,point-origin)/dot(normal,direction));}
float3 barycentric(float3 point,float3 a,float3 b,float3 c){
 float3 e0=b-a,e1=c-a,e2=point-a;float d00=dot(e0,e0),d01=dot(e0,e1),d11=dot(e1,e1),d20=dot(e2,e0),d21=dot(e2,e1);float den=1.0f/(d00*d11-d01*d01);float v=(d11*d20-d01*d21)*den,w=(d00*d21-d01*d20)*den;return float3(1-v-w,v,w);
}
float3x3 tangent_frame(float3 n,float3 t,float3 bt,float3 e1,float3 e2,float2 d1,float2 d2,thread uint& repaired){
 repaired=0;
 if(!all(isfinite(t))||!all(isfinite(bt))||dot(t,t)<1e-12||dot(bt,bt)<1e-12){
  repaired=1;float det=d1.x*d2.y-d1.y*d2.x;
  if(abs(det)>1e-10){t=(e1*d2.y-e2*d1.y)/det;bt=(e2*d1.x-e1*d2.x)/det;}
  else {t=cross(abs(n.y)<.99f?float3(0,1,0):float3(1,0,0),n);bt=cross(n,t);}
 }
 return float3x3(normalize(t),normalize(bt),normalize(n));
}
Surface surface_at(uint instance,uint primitive,float2 hit_bary,float3 origin,float3 direction,float distance,uint2 pixel,
                   device const Vertex* vertices,device const MeshInfo* mesh,constant MaterialTextures& textures,constant Params& p){
 uint base=mesh[instance].base+primitive*3,material=mesh[instance].material;
 Vertex a=vertices[base],b=vertices[base+1],c=vertices[base+2];float3 weights=float3(1-hit_bary.x-hit_bary.y,hit_bary);
 float3 pa=a.position,pb=b.position,pc=c.position;float2 ua=a.uv,ub=b.uv,uc=c.uv;
 float2 uv=ua*weights.x+ub*weights.y+uc*weights.z;float3 position=origin+direction*distance;
 float3 n=normalize(float3(a.normal)*weights.x+float3(b.normal)*weights.y+float3(c.normal)*weights.z);
 float3 t=float3(a.tangent)*weights.x+float3(b.tangent)*weights.y+float3(c.tangent)*weights.z;
 float3 bt=float3(a.bitangent)*weights.x+float3(b.bitangent)*weights.y+float3(c.bitangent)*weights.z;
 uint repaired=0;
 // Preserve valid original frames. For the asset's invalid frames, use its
 // triangle/UV derivatives rather than permitting NaN shaded surfaces.
 float3x3 frame=tangent_frame(n,t,bt,pb-pa,pc-pa,ub-ua,uc-ua,repaired);
 float3 plane_normal=normalize(cross(pc-pa,pb-pa));
 float3 px=plane_point(position,plane_normal,p.eye.xyz,camera_direction(float2(pixel)+float2(1,0),p));
 float3 py=plane_point(position,plane_normal,p.eye.xyz,camera_direction(float2(pixel)+float2(0,1),p));
 float3 bx=barycentric(px,pa,pb,pc),by=barycentric(py,pa,pb,pc);
 float2 dx=ua*bx.x+ub*bx.y+uc*bx.z-uv,dy=ua*by.x+ub*by.y+uc*by.z-uv;
 if(!all(isfinite(dx))||!all(isfinite(dy))){dx=0;dy=0;}
 float3 diffuse=textures.diffuse[material].sample(wrap_linear,uv,gradient2d(dx,dy)).rgb;
 float3 map_normal=textures.normal[material].sample(wrap_linear,uv,gradient2d(dx,dy)).rgb*2-1;
 float gloss=1.0f+127.0f*length(map_normal);map_normal=normalize(map_normal);float3 normal=normalize(frame*map_normal);
 // The sample identifies the ground as reflective. Keep the diagnostic port's
 // floor mask explicit; use its actual material specular texture for strength.
 float spec=textures.specular[material].sample(wrap_linear,uv,gradient2d(dx,dy)).g;
 return {position,normal,diffuse,material==10?spec:0.0f,spec,gloss,repaired};
}
float3 sun_direction(){float theta=-.5f,phi=.75f*3.14159f*.5f;return normalize(float3(cos(theta)*cos(phi),sin(phi),sin(theta)*cos(phi)));}
float3 lit_colour(Surface surface,float3 incident,float visibility,bool specular_enabled){
 float3 sun=sun_direction();// Raster-style material shading and upstream DiffuseHit use opposite view conventions.
 float3 view=specular_enabled?normalize(incident):-normalize(incident),half_vector=normalize(sun-view);
 float fresnel=pow(1-saturate(dot(sun,half_vector)),5.0f);
 float3 diffuse=surface.diffuse*(1-fresnel),specular=mix(float3(.56),float3(1),fresnel);
 float spec=specular_enabled?surface.specular*pow(saturate(dot(half_vector,surface.normal)),surface.gloss)*(surface.gloss+2)/8:0;
 return surface.diffuse*.1f+visibility*4.0f*saturate(dot(surface.normal,sun))*(diffuse+spec*specular);
}
float3 display_colour(float3 linear){
 // MiniEngine TM_Stanard with its initial fixed exposure 2; adaptive exposure,
 // bloom and temporal postprocessing remain separate, explicitly unimplemented.
 float3 hdr=max(linear*2.0f,float3(0));float3 shoulder=hdr*sqrt(hdr);float3 sdr=shoulder/(shoulder+sqrt(4.0f/27.0f));
 return select(1.055f*pow(sdr,float3(1.0f/2.4f))-.055f,sdr*12.92f,sdr<=.0031308f);
}
