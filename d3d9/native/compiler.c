/* Original MIT adapter to the separately built LGPL vkd3d-shader library.
 * Shader math is compiled by pinned upstream, not replaced by a canned shader.
 * Usage: compiler input.d3dbc output.spv reflection.json (all paths explicit). */
#include <vkd3d_shader.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static void quote(FILE *f,const char *s){fputc('"',f);for(;*s;s++){unsigned char c=(unsigned char)*s;if(c=='"'||c=='\\'){fputc('\\',f);fputc(c,f);}else if(c<32||c>=127)fprintf(f,"\\u%04x",c);else fputc(c,f);}fputc('"',f);}
static void signature(FILE *f,const struct vkd3d_shader_signature *s){
 fputc('[',f);for(unsigned int i=0;i<s->element_count;i++){
  const struct vkd3d_shader_signature_element *e=s->elements+i;if(i)fputc(',',f);
  fputs("{\"name\":",f);quote(f,e->semantic_name);
  fprintf(f,",\"index\":%u,\"register\":%u,\"system\":%u,\"mask\":%u,\"used_mask\":%u,\"component\":%u}",e->semantic_index,e->register_index,(unsigned)e->sysval_semantic,e->mask,e->used_mask,(unsigned)e->component_type);
 }fputc(']',f);
}
int main(int argc,char **argv){
 if(argc!=4&&argc!=5){fprintf(stderr,"expected input.d3dbc output.spv reflection.json [following-pixel.d3dbc]\n");return 2;}
 FILE *in=fopen(argv[1],"rb");if(!in){perror("shader input");return 2;}
 if(fseek(in,0,SEEK_END)){fclose(in);return 2;}long length=ftell(in);rewind(in);
 if(length<8||length>1024*1024||length%4){fprintf(stderr,"invalid bounded legacy bytecode size\n");fclose(in);return 2;}
 unsigned char *bytes=malloc((size_t)length);if(!bytes){fclose(in);return 2;}
 if(fread(bytes,1,(size_t)length,in)!=(size_t)length){fclose(in);free(bytes);return 2;}fclose(in);
 unsigned int tag=(unsigned int)bytes[2]|((unsigned int)bytes[3]<<8);
 if(tag!=0xfffe&&tag!=0xffff){fprintf(stderr,"not legacy vertex/pixel bytecode\n");free(bytes);return 2;}
 unsigned int group=tag==0xfffe?0:1;
 struct vkd3d_shader_scan_descriptor_info descriptors={.type=VKD3D_SHADER_STRUCTURE_TYPE_SCAN_DESCRIPTOR_INFO};
 struct vkd3d_shader_scan_signature_info sig={.type=VKD3D_SHADER_STRUCTURE_TYPE_SCAN_SIGNATURE_INFO,.next=&descriptors};
 struct vkd3d_shader_compile_info info={.type=VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO,.next=&sig,.source={bytes,(size_t)length},.source_type=VKD3D_SHADER_SOURCE_D3D_BYTECODE,.target_type=VKD3D_SHADER_TARGET_SPIRV_BINARY,.log_level=VKD3D_SHADER_LOG_WARNING,.source_name=group?"alkyhol_pixel.d3dbc":"alkyhol_vertex.d3dbc"};
 char *messages=NULL;int result=vkd3d_shader_scan(&info,&messages);
 if(messages){fputs(messages,stderr);vkd3d_shader_free_messages(messages);messages=NULL;}
 if(result<0){free(bytes);return 3;}
 if(descriptors.descriptor_count>64){fprintf(stderr,"legacy descriptor budget exceeded\n");return 3;}
 struct vkd3d_shader_resource_binding bindings[64]={0};
 for(unsigned int i=0;i<descriptors.descriptor_count;i++){
  const struct vkd3d_shader_descriptor_info *d=descriptors.descriptors+i;
  if(d->register_space||d->count!=1){fprintf(stderr,"unsupported descriptor space/count\n");return 3;}
  unsigned int binding,flags;
  if(d->type==VKD3D_SHADER_DESCRIPTOR_TYPE_CBV&&d->register_index<=2){binding=d->register_index;flags=VKD3D_SHADER_BINDING_FLAG_BUFFER;}
  else if(d->type==VKD3D_SHADER_DESCRIPTOR_TYPE_SRV&&d->register_index<16){binding=16+d->register_index;flags=VKD3D_SHADER_BINDING_FLAG_IMAGE;}
  else if(d->type==VKD3D_SHADER_DESCRIPTOR_TYPE_SAMPLER&&d->register_index<16){binding=32+d->register_index;flags=0;}
  else{fprintf(stderr,"unsupported legacy descriptor\n");return 3;}
  bindings[i]=(struct vkd3d_shader_resource_binding){.type=d->type,.register_space=0,.register_index=d->register_index,.shader_visibility=VKD3D_SHADER_VISIBILITY_ALL,.flags=flags,.binding={.set=group,.binding=binding,.count=1}};
 }
 struct vkd3d_shader_spirv_target_info target={.type=VKD3D_SHADER_STRUCTURE_TYPE_SPIRV_TARGET_INFO,.entry_point="main",.environment=VKD3D_SHADER_SPIRV_ENVIRONMENT_VULKAN_1_0};
 struct vkd3d_shader_interface_info interface={.type=VKD3D_SHADER_STRUCTURE_TYPE_INTERFACE_INFO,.next=&target,.bindings=bindings,.binding_count=descriptors.descriptor_count};
 struct vkd3d_shader_varying_map map[12];
 struct vkd3d_shader_varying_map_info map_info={.type=VKD3D_SHADER_STRUCTURE_TYPE_VARYING_MAP_INFO,.varying_map=map};
 struct vkd3d_shader_scan_signature_info following={.type=VKD3D_SHADER_STRUCTURE_TYPE_SCAN_SIGNATURE_INFO};
 unsigned char *following_bytes=NULL;
 if(argc==5){
  if(group!=0){fprintf(stderr,"only vertex shaders have a following pixel stage\n");return 2;}
  FILE *f=fopen(argv[4],"rb");if(!f)return 2;
  if(fseek(f,0,SEEK_END)){fclose(f);return 2;}long n=ftell(f);rewind(f);
  if(n<8||n>1024*1024||n%4){fclose(f);return 2;}
  following_bytes=malloc((size_t)n);if(!following_bytes){fclose(f);return 2;}
  if(fread(following_bytes,1,(size_t)n,f)!=(size_t)n){fclose(f);return 2;}fclose(f);
  if(following_bytes[2]!=0xff||following_bytes[3]!=0xff){fprintf(stderr,"following stage is not a pixel shader\n");return 2;}
  struct vkd3d_shader_compile_info next_info=info;next_info.next=&following;next_info.source=(struct vkd3d_shader_code){following_bytes,(size_t)n};next_info.source_name=argv[4];messages=NULL;
  result=vkd3d_shader_scan(&next_info,&messages);if(messages){fputs(messages,stderr);vkd3d_shader_free_messages(messages);messages=NULL;}
  if(result<0||following.input.element_count>12){fprintf(stderr,"invalid following stage signature\n");return 3;}
  vkd3d_shader_build_varying_map(&sig.output,&following.input,&map_info.varying_count,map);target.next=&map_info;
 }
 info.next=&interface;struct vkd3d_shader_code output={0};result=vkd3d_shader_compile(&info,&output,&messages);
 if(messages){fputs(messages,stderr);vkd3d_shader_free_messages(messages);}
 if(result<0){fprintf(stderr,"legacy compilation failed: %d\n",result);return 3;}
 FILE *spv=fopen(argv[2],"wb");if(!spv){perror("SPIR-V output");return 2;}
 if(fwrite(output.code,1,output.size,spv)!=output.size){fclose(spv);return 2;}if(fclose(spv))return 2;
 FILE *json=fopen(argv[3],"wb");if(!json){perror("reflection output");return 2;}
 fputs("{\"version\":1,\"compiler\":",json);quote(json,vkd3d_shader_get_version(NULL,NULL));fprintf(json,",\"stage\":%u,\"inputs\":",group);signature(json,&sig.input);fputs(",\"outputs\":",json);signature(json,&sig.output);fputs(",\"descriptors\":[",json);
 for(unsigned int i=0;i<descriptors.descriptor_count;i++){
  const struct vkd3d_shader_descriptor_info *d=descriptors.descriptors+i;if(i)fputc(',',json);
  fprintf(json,"{\"type\":%u,\"register\":%u,\"dimension\":%u,\"group\":%u,\"binding\":%u}",(unsigned)d->type,d->register_index,(unsigned)d->resource_type,group,bindings[i].binding.binding);
 }
 fputs("]}\n",json);int failed=fclose(json);
 vkd3d_shader_free_shader_code(&output);vkd3d_shader_free_scan_signature_info(&sig);vkd3d_shader_free_scan_descriptor_info(&descriptors);vkd3d_shader_free_scan_signature_info(&following);free(following_bytes);free(bytes);return failed?2:0;
}
