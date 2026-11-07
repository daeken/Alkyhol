// Microsoft MiniEngine RayGen/Hit/Miss primary barycentric path; MIT, see licenses/.
#include <metal_stdlib>
#include <metal_raytracing>
using namespace metal;using namespace raytracing;
struct Vertex {packed_float3 position;packed_float2 uv;packed_float3 normal,tangent,bitangent;};
struct Params {float4 eye,right,up,forward;uint width,height,mode,padding;};
kernel void render_scene(instance_acceleration_structure scene [[buffer(0)]],constant Params& p [[buffer(1)]],
                         device atomic_uint* counts [[buffer(2)]],device const Vertex* vertices [[buffer(3)]],device const uint* offsets [[buffer(4)]],texture2d<float,access::write> output [[texture(0)]],
                         uint2 pixel [[thread_position_in_grid]]) {
 if(pixel.x>=p.width||pixel.y>=p.height)return;
 float2 screen=(float2(pixel)+.5f)/float2(p.width,p.height)*2-1;screen.y=-screen.y;
 ray r;r.origin=p.eye.xyz;r.direction=normalize(p.forward.xyz+p.right.xyz*screen.x*(float(p.width)/p.height)*tan(M_PI_F/8.0f)+p.up.xyz*screen.y*tan(M_PI_F/8.0f));r.min_distance=0;r.max_distance=FLT_MAX;
 intersector<triangle_data,instancing> tracer;tracer.assume_geometry_type(geometry_type::triangle);tracer.force_opacity(forced_opacity::opaque);tracer.accept_any_intersection(false);
 tracer.set_triangle_cull_mode(triangle_cull_mode::back);
 tracer.set_triangle_front_facing_winding(winding::clockwise);
 auto hit=tracer.intersect(r,scene,1);
 if(hit.type==intersection_type::none){output.write(float4(0,0,0,1),pixel);atomic_fetch_add_explicit(&counts[34],1,memory_order_relaxed);}
 else {
  float3 colour=float3(hit.triangle_barycentric_coord,1);
  if(p.mode!=2) {
   uint base=offsets[hit.instance_id]+hit.primitive_id*3;float2 b=hit.triangle_barycentric_coord;
   float3 n0=vertices[base].normal,n1=vertices[base+1].normal,n2=vertices[base+2].normal;
   float3 normal=normalize(n0+b.x*(n1-n0)+b.y*(n2-n0));
   if(p.mode==0)colour=float3(.72)*(.4f+.6f*saturate(dot(normal,-r.direction)));
   else if(p.mode==8)colour=normal*.5f+.5f;
   else if(p.mode==9)colour=float3(1.0f/(1.0f+hit.distance/1000.0f));
  }
  output.write(float4(colour,1),pixel);atomic_fetch_add_explicit(&counts[hit.instance_id],1,memory_order_relaxed);
 }
}
struct ScreenVertex {float4 position [[position]];float2 uv;};
vertex ScreenVertex screen_vertex(uint id [[vertex_id]]) {float2 p=float2((id<<1)&2,id&2);return {float4(p*2-1,0,1),float2(p.x,1-p.y)};}
fragment float4 screen_fragment(ScreenVertex in [[stage_in]],texture2d<float> image [[texture(0)]]) {constexpr sampler s(coord::normalized,address::clamp_to_edge,filter::linear);return image.sample(s,in.uv);}
