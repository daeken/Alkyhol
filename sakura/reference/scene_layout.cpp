// Placement adaptation from Microsoft DirectX-Graphics-Samples, revision in ../upstream.json.
// Copyright (c) Microsoft. MIT, see ../licenses/Microsoft-MIT.txt.
// Independent native MSVC reference: no Rust/Alkyhol code or exported fixtures.
#include <array>
#include <bit>
#include <cstdint>
#include <cstdio>
#include <random>
#include <vector>
struct Instance { uint32_t mesh,material; float scale,x,y,z; };
int main(int argc,char**argv) {
 if(argc!=2)return 2;
 std::vector<Instance> out;
 std::mt19937 gen(42), genBush(42);
 std::uniform_real_distribution<float> randomOffset(-0.1f,0.7f),randomOffsetBush(0.7f,0.75f);
 for(int x=-10;x<=10;++x)for(int z=-10;z<=10;++z)out.push_back({0,0,2.0f,static_cast<float>(x)*4.0f+10.0f,0.0f,static_cast<float>(z)*4.0f});
 for(int x=-15;x<=15;++x)for(int z=-15;z<=15;++z) {float ry=randomOffset(gen);out.push_back({0,1,0.01f,0.5f+static_cast<float>(x)*0.1f,1.7f+ry,static_cast<float>(z)*0.1f});}
 std::vector<std::array<float,3>> trees;
 for(int x=-15;x<=15;++x)for(int z=-15;z<=15;++z) {
  float rx=randomOffset(gen),ry=randomOffset(gen),rz=randomOffset(gen);
  float px=static_cast<float>(x)*1.9f+rx/6,py=2.0f-ry/3,pz=static_cast<float>(z)*1.9f+rz/6;
  trees.push_back({px,py,pz});out.push_back({1,2,35.0f,px,py,pz});
 }
 for(auto p:trees)out.push_back({2,3,35.0f,p[0],p[1],p[2]});
 for(int x=-50;x<=50;++x)for(int z=-50;z<=50;++z) {
  float rx=randomOffset(gen),ry=randomOffsetBush(genBush),rz=randomOffset(gen);
  out.push_back({3,4,2.1f,static_cast<float>(x)*0.2f+rx,1.7f+ry/3,static_cast<float>(z)*0.2f+rz});
 }
 FILE* f=nullptr;
 #ifdef _MSC_VER
 if(fopen_s(&f,argv[1],"wb")!=0)return 3;
 #else
 f=fopen(argv[1],"wb");if(!f)return 3;
 #endif
 for(const auto&i:out)if(fprintf(f,"%08x%08x%08x%08x%08x%08x\n",i.mesh,i.material,std::bit_cast<uint32_t>(i.scale),std::bit_cast<uint32_t>(i.x),std::bit_cast<uint32_t>(i.y),std::bit_cast<uint32_t>(i.z))!=49)return 4;
 if(fclose(f)!=0)return 5;
 printf("Native placement reference: %zu instances\n",out.size());return 0;
}
