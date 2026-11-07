float shadow_visibility(Surface surface,instance_acceleration_structure scene,device atomic_uint* counts){
 ray r;r.origin=surface.position;r.direction=sun_direction();r.min_distance=.1f;r.max_distance=FLT_MAX;
 intersector<triangle_data,instancing> shadow;shadow.assume_geometry_type(geometry_type::triangle);shadow.force_opacity(forced_opacity::opaque);shadow.accept_any_intersection(true);shadow.set_triangle_cull_mode(triangle_cull_mode::none);
 auto hit=shadow.intersect(r,scene,1);atomic_fetch_add_explicit(&counts[35],1,memory_order_relaxed);
 if(hit.type!=intersection_type::none){atomic_fetch_add_explicit(&counts[36],1,memory_order_relaxed);return 0;}return 1;
}
kernel void render_scene(instance_acceleration_structure scene [[buffer(0)]],constant Params& p [[buffer(1)]],
                         device atomic_uint* counts [[buffer(2)]],device const Vertex* vertices [[buffer(3)]],device const MeshInfo* mesh [[buffer(4)]],
                         constant MaterialTextures& textures [[buffer(5)]],device const float4* camera_rays [[buffer(6)]],texture2d<float,access::write> output [[texture(0)]],uint2 pixel [[thread_position_in_grid]]){
 if(pixel.x>=p.width||pixel.y>=p.height)return;
 uint ray_offset=(pixel.y*p.width+pixel.x)*2;float4 ray_origin=camera_rays[ray_offset],ray_direction=camera_rays[ray_offset+1];
 ray r;r.origin=ray_origin.xyz;r.direction=ray_direction.xyz;r.min_distance=ray_origin.w;r.max_distance=ray_direction.w;
 intersector<triangle_data,instancing> tracer;tracer.assume_geometry_type(geometry_type::triangle);tracer.force_opacity(forced_opacity::opaque);tracer.accept_any_intersection(false);tracer.set_triangle_cull_mode(triangle_cull_mode::back);tracer.set_triangle_front_facing_winding(winding::clockwise);
 auto hit=tracer.intersect(r,scene,1);
 if(hit.type==intersection_type::none){output.write(float4(0,0,0,1),pixel);atomic_fetch_add_explicit(&counts[34],1,memory_order_relaxed);return;}
 atomic_fetch_add_explicit(&counts[hit.instance_id],1,memory_order_relaxed);
 float3 colour=float3(hit.triangle_barycentric_coord,1);
 if(p.mode==2){output.write(float4(colour,1),pixel);return;}
 uint base=mesh[hit.instance_id].base+hit.primitive_id*3;float2 b=hit.triangle_barycentric_coord;
 float3 n0=vertices[base].normal,n1=vertices[base+1].normal,n2=vertices[base+2].normal;
 float3 normal=normalize(n0+b.x*(n1-n0)+b.y*(n2-n0));
 if(p.mode==0)colour=float3(.72)*(.4f+.6f*saturate(dot(normal,-r.direction)));
 else if(p.mode==8)colour=normal*.5f+.5f;
 else if(p.mode==9)colour=float3(1.0f/(1.0f+hit.distance/1000.0f));
 else {
  Surface surface=surface_at(hit.instance_id,hit.primitive_id,b,r.origin,r.direction,hit.distance,pixel,vertices,mesh,textures,p);
  atomic_fetch_add_explicit(&counts[39],surface.repaired,memory_order_relaxed);
  if(!all(isfinite(surface.normal))||!all(isfinite(surface.diffuse))||!isfinite(surface.reflectivity)){atomic_fetch_add_explicit(&counts[41],1,memory_order_relaxed);output.write(float4(1,0,1,1),pixel);return;}
  float visibility=p.mode==5?1.0f:shadow_visibility(surface,scene,counts);
  if(p.mode==4){output.write(float4(float3(visibility),1),pixel);return;}
  colour=lit_colour(surface,r.direction,visibility,p.mode!=6);
  if((p.mode==7||p.mode==3)&&surface.reflectivity>0){
   ray reflection;reflection.origin=surface.position+r.direction*.1f;reflection.direction=normalize(reflect(r.direction,surface.normal));reflection.min_distance=0;reflection.max_distance=FLT_MAX;
   auto secondary=tracer.intersect(reflection,scene,1);atomic_fetch_add_explicit(&counts[37],1,memory_order_relaxed);
   if(secondary.type!=intersection_type::none){
    atomic_fetch_add_explicit(&counts[38],1,memory_order_relaxed);
    if(secondary.instance_id==hit.instance_id&&secondary.distance<.2f)atomic_fetch_add_explicit(&counts[40],1,memory_order_relaxed);
    if(p.mode==3){output.write(float4(secondary.triangle_barycentric_coord,1,1),pixel);return;}
    Surface reflected=surface_at(secondary.instance_id,secondary.primitive_id,secondary.triangle_barycentric_coord,reflection.origin,reflection.direction,secondary.distance,pixel,vertices,mesh,textures,p);
    atomic_fetch_add_explicit(&counts[39],reflected.repaired,memory_order_relaxed);
    if(!all(isfinite(reflected.normal))||!all(isfinite(reflected.diffuse))){atomic_fetch_add_explicit(&counts[41],1,memory_order_relaxed);output.write(float4(1,0,1,1),pixel);return;}
    colour+=surface.reflectivity*lit_colour(reflected,reflection.direction,shadow_visibility(reflected,scene,counts),false);
   }
  }
  colour=display_colour(colour);
 }
 if(!all(isfinite(colour))){atomic_fetch_add_explicit(&counts[41],1,memory_order_relaxed);output.write(float4(1,0,1,1),pixel);return;}
 output.write(float4(colour,1),pixel);
}
struct ScreenVertex {float4 position [[position]];float2 uv;};
vertex ScreenVertex screen_vertex(uint id [[vertex_id]]) {float2 p=float2((id<<1)&2,id&2);return {float4(p*2-1,0,1),float2(p.x,1-p.y)};}
fragment float4 screen_fragment(ScreenVertex in [[stage_in]],texture2d<float> image [[texture(0)]]) {constexpr sampler s(coord::normalized,address::clamp_to_edge,filter::linear);return image.sample(s,in.uv);}
