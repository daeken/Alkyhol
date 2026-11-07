// Faithful SORTMODE_OFF port of Microsoft's Raytracing.hlsl, pinned in upstream.json.
// Copyright (c) Microsoft. MIT; see licenses/Microsoft-MIT.txt.
// Star Nest formula: Copyright (c) Pablo Roman Andrioli. MIT; see licenses/Star-Nest-MIT.txt.
#include <metal_stdlib>
#include <metal_raytracing>
using namespace metal;
using namespace raytracing;
struct Vertex { packed_float3 position; packed_float3 normal; packed_float2 uv; };
struct Params {
    float4 eye, right, up, forward, light;
    uint width, height, leaf_vertices, debug;
};
constexpr sampler linear_wrap(coord::normalized,address::repeat,filter::linear,mip_filter::none);
constant uint cube_indices[36]={3,1,0,2,1,3,6,4,5,7,4,6,11,9,8,10,9,11,14,12,13,15,12,14,19,17,16,18,17,19,22,20,21,23,20,22};
float3 interpolate_normal(device const Vertex* v,uint3 indices,float2 b) {
    float3 a=v[indices.x].normal,c=v[indices.y].normal,d=v[indices.z].normal;
    return a+b.x*(c-a)+b.y*(d-a);
}
float star_density(float3 direction) {
    float3 sky=direction*10.0f;float density=0,fade=1;
    for(int s=0;s<200;++s) {
        float3 p=sky+direction*(s*0.05f);p=abs(fmod(p,2.0f)-1.0f);
        float pa=0,a=0;
        for(int i=0;i<50;++i) {float inv=1.0f/dot(p,p);p=abs(p)*inv-0.53f;float len=length(p);a+=abs(len-pa);pa=len;}
        density+=a*fade;fade*=0.9f;
    }
    return smoothstep(0.1f,0.9f,saturate(density/(200*50*0.12f)));
}
float4 miss_colour(float3 direction,float3 light) {
    float3 d=normalize(direction),sky=d*10.0f;
    float density=star_density(d);float3 glow=mix(float3(.4,.6,1),float3(1,.6,.8),density);
    float ld=saturate(dot(d,normalize(light-sky)));glow*=mix(.8f+.2f*ld,4.0f,density);
    return float4(mix(float3(.05,.02,.08),glow,density*1.3f),1);
}
// Explicit two-level stack: identical maximum recursion depth to the source, no MSL recursion.
float4 radiance(ray r,instance_acceleration_structure scene,
               device const Vertex* cube,device const Vertex* trunk,device const Vertex* leaves,device const Vertex* bush,
               texture2d<float> trunk_tex,texture2d<float> bush_tex,constant Params& p,
               thread uint& first_material, device atomic_uint* counts) {
    float3 multiplier=1,addition=0;float alpha_scale=1;
    intersector<triangle_data,instancing> tracer;tracer.assume_geometry_type(geometry_type::triangle);tracer.force_opacity(forced_opacity::opaque);
    for(uint depth=1;depth<=2;++depth) {
        r.min_distance=.001f;r.max_distance=10000;
        auto hit=tracer.intersect(r,scene,1);
        if(hit.type==intersection_type::none) {if(depth==2)atomic_fetch_add_explicit(&counts[9],1,memory_order_relaxed);if(depth==1)first_material=5;float4 colour=miss_colour(r.direction,p.light.xyz);return float4(addition+multiplier*colour.rgb,alpha_scale*colour.a);}
        if(depth==2)atomic_fetch_add_explicit(&counts[8],1,memory_order_relaxed);
        uint id=hit.instance_id,primitive=hit.primitive_id;
        uint material=id<441?0:id<1402?1:id<2363?2:id<3324?3:4;
        if(depth==1) first_material=material;
        float3 position=r.origin+r.direction*hit.distance;
        float2 b=hit.triangle_barycentric_coord;
        uint base=primitive*3;uint3 indices=uint3(base,base+1,base+2);
        float3 normal;
        if(material==0)normal=interpolate_normal(cube,indices,b);
        else if(material==1)normal=interpolate_normal(trunk,uint3(cube_indices[base],cube_indices[base+1],cube_indices[base+2]),b);
        else if(material==2) {
            // Upstream reads IndicesLeaves with a trunk PrimitiveIndex. Its sequential indices
            // are explicit zero when the leaf index-buffer read is outside its declared bounds.
            if(base+2>=p.leaf_vertices)indices=uint3(0);
            normal=interpolate_normal(trunk,indices,b);
        } else if(material==3)normal=interpolate_normal(leaves,indices,b);
        else normal=interpolate_normal(bush,indices,b);
        float3 colour;
        if(material==0) {
            float3 sampled=trunk_tex.sample(linear_wrap,fract(position.xz),level(0)).rgb;
            if(all(sampled<float3(.1))&&depth<2) {
                atomic_fetch_add_explicit(&counts[6],1,memory_order_relaxed);
                float cosi=saturate(dot(-r.direction,normal));float3 fresnel=sampled+(1-sampled)*pow(1-cosi,5.0f);
                addition+=multiplier*sampled*(1-fresnel)*1.7f;multiplier*=fresnel*1.7f;
                r.origin=position+normal*.001f;r.direction=reflect(r.direction,normal);continue;
            }
            colour=sampled;
        } else if(material==1) {
            if(depth==1)atomic_fetch_add_explicit(&counts[7],1,memory_order_relaxed);
            multiplier*=.5f;alpha_scale*=.5f;
            // The floor ignores its secondary payload alpha; a primary reflective cube uses it.
            if(depth==2)return float4(addition,first_material==0?1.0f:0.0f);
            r.origin=position+normal*.5f;r.direction=reflect(r.direction,normal);continue;
        } else if(material==2) {
            float3 weights=normalize(max(abs(normal),float3(.00001)));
            colour=(trunk_tex.sample(linear_wrap,position.yz*.5f,level(0)).rgb*weights.x+
                    trunk_tex.sample(linear_wrap,position.xz*.5f,level(0)).rgb*weights.y+
                    trunk_tex.sample(linear_wrap,position.xy*.5f,level(0)).rgb*weights.z)*.5f;
        } else {
            float ndotl=saturate(dot(normal,normalize(p.light.xyz-position)));
            if(material==3)colour=primitive%3==0?float3(.8,.7,.9):primitive%3==1?float3(1,.8,.9):float3(.85,.4,.6);
            else {
                float2 a=bush[indices.x].uv,c=bush[indices.y].uv,d=bush[indices.z].uv;
                colour=bush_tex.sample(linear_wrap,a+b.x*(c-a)+b.y*(d-a),level(0)).rgb;
            }
            colour*=float3(1.2,1,1)*ndotl;
        }
        return float4(addition+multiplier*colour,alpha_scale);
    }
    return float4(addition,0);
}
kernel void render_sakura(instance_acceleration_structure scene [[buffer(0)]],
                          device const Vertex* cube [[buffer(1)]],device const Vertex* trunk [[buffer(2)]],
                          device const Vertex* leaves [[buffer(3)]],device const Vertex* bush [[buffer(4)]],
                          constant Params& p [[buffer(5)]],device atomic_uint* counts [[buffer(6)]],
                          texture2d<float> trunk_tex [[texture(0)]],texture2d<float> bush_tex [[texture(1)]],
                          texture2d<float,access::write> output [[texture(2)]],uint2 pixel [[thread_position_in_grid]]) {
    if(pixel.x>=p.width||pixel.y>=p.height)return;
    float2 screen=(float2(pixel)+.5f)/float2(p.width,p.height)*2.0f-1.0f;screen.y=-screen.y;
    float tan_half_fov=tan(M_PI_F/8.0f);
    ray r;r.origin=p.eye.xyz;r.direction=normalize(p.forward.xyz+p.right.xyz*screen.x*(float(p.width)/p.height)*tan_half_fov+p.up.xyz*screen.y*tan_half_fov);
    uint material=5;float4 colour=radiance(r,scene,cube,trunk,leaves,bush,trunk_tex,bush_tex,p,material,counts);
    atomic_fetch_add_explicit(&counts[material],1,memory_order_relaxed);
    output.write(colour,pixel);
}
struct ScreenVertex { float4 position [[position]];float2 uv; };
vertex ScreenVertex screen_vertex(uint id [[vertex_id]]) {
    float2 p=float2((id<<1)&2,id&2);
    return {float4(p*2-1,0,1),float2(p.x,1-p.y)};
}
fragment float4 screen_fragment(ScreenVertex in [[stage_in]],texture2d<float> image [[texture(0)]]) {
    constexpr sampler s(coord::normalized,address::clamp_to_edge,filter::linear);return image.sample(s,in.uv);
}
