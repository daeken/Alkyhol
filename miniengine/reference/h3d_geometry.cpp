// Independent native ABI/geometry readback for Microsoft's ModelH3D serialization.
// Struct fields follow pinned MiniEngine/Model/ModelH3D.h; MIT, see ../licenses/.
#include <array>
#include <cstdint>
#include <fstream>
#include <vector>
#include <cstring>
struct alignas(16) Vector3 {float x,y,z,padding;};
struct Box {Vector3 minimum,maximum;};
struct Header {uint32_t meshes,materials,vertexBytes,indexBytes,depthBytes;Box bounds;};
struct Attrib {uint16_t offset,normalized,components,format;};
struct Mesh {Box bounds;uint32_t material,enabled,depthEnabled,stride,depthStride;Attrib attrib[16],depthAttrib[16];uint32_t vertexOffset,vertexCount,indexOffset,indexCount,depthOffset,depthCount;};
struct alignas(16) Material {float colours[20];float opacity,shininess,strength;char paths[6][128],name[128];};
static_assert(sizeof(Header)==64&&sizeof(Mesh)==336&&sizeof(Material)==992);
int main(int argc,char**argv){
 if(argc!=3)return 2;std::ifstream input(argv[1],std::ios::binary);Header h{};input.read(reinterpret_cast<char*>(&h),sizeof(h));if(!input||h.meshes!=34||h.materials!=27)return 3;
 std::vector<Mesh> meshes(h.meshes);input.read(reinterpret_cast<char*>(meshes.data()),sizeof(Mesh)*meshes.size());input.seekg(sizeof(Material)*h.materials,std::ios::cur);
 std::vector<char> vertices(h.vertexBytes),indices(h.indexBytes);input.read(vertices.data(),vertices.size());input.read(indices.data(),indices.size());if(!input)return 4;
 std::ofstream output(argv[2],std::ios::binary);
 for(const auto&m:meshes){
  if(m.material>=h.materials||m.stride!=56||m.indexCount%3!=0||uint64_t(m.vertexOffset)+uint64_t(m.vertexCount)*56>vertices.size()||uint64_t(m.indexOffset)+uint64_t(m.indexCount)*2>indices.size())return 5;
  output.write(reinterpret_cast<const char*>(&m.material),4);output.write(reinterpret_cast<const char*>(&m.indexCount),4);
  for(uint32_t i=0;i<m.indexCount;++i){uint16_t index;std::memcpy(&index,indices.data()+m.indexOffset+i*2,2);if(index>=m.vertexCount)return 6;output.write(vertices.data()+m.vertexOffset+index*56,56);}
 }
 return output?0:7;
}
