// Copyright (c) Alkyhol contributors. MIT licensed.
#include "Windows.h"
#include <fstream>
#include <malloc.h>
#include <cassert>
#include <cstring>
#include <string>
int main(){
    std::ifstream default_in;std::ofstream default_out;
    wchar_t formatted[128]={};assert(swprintf_s(formatted,L"%s::closesthit",L"Bark")==16&&std::wcscmp(formatted,L"Bark::closesthit")==0);
    assert(swprintf_s(formatted,L"%.2s %S %ws %%",L"abcd","ansi",L"wide")==14&&std::wcscmp(formatted,L"ab ansi wide %")==0);
    assert(alkyhol_swprintf(formatted,2,L"%s",L"too long")==-1&&formatted[0]==0);
    std::wstring fmt;assert(!alkyhol_wide_format(L"%",fmt)&&errno==EINVAL);assert(!alkyhol_wide_format(L"%n",fmt)&&errno==ENOSYS);
    const std::wstring path=L".\\alkyhol_\u03bb_crt.tmp";
    const unsigned char data[]={0,1,2,3,250,251,252,253,254,255};
    {std::ofstream out(path,std::ios::binary);assert(out);out.write(reinterpret_cast<const char*>(data),sizeof(data));assert(out.good());}
    {std::ifstream in(path,std::ios::binary);unsigned char read[sizeof(data)]={};in.read(reinterpret_cast<char*>(read),sizeof(read));assert(in.gcount()==sizeof(read));assert(std::memcmp(read,data,sizeof(read))==0);}
    _stat64 stat{};assert(_wstat64(path.c_str(),&stat)==0);assert(stat.st_size==sizeof(data));assert(stat.st_mtime>0);
    assert(std::remove(alkyhol_path(path).c_str())==0);assert(_wstat64(path.c_str(),&stat)==-1);
    char text[8]={};assert(strcpy_s(text,"abc")==0&&std::strcmp(text,"abc")==0);
    assert(strcpy_s(text,"12345678")==ERANGE&&text[0]==0);
    FILE* file=std::tmpfile();assert(file);std::fputs("42 hello skip the rest of this line\n",file);std::rewind(file);
    int value=0;assert(fscanf_s(file,"%d",&value)==1&&value==42);
    assert(fscanf_s(file,"%s",text,sizeof(text))==1&&std::strcmp(text,"hello")==0);
    char line[64]={};assert(fscanf_s(file," %*s %[^\n]",line,sizeof(line))==1&&std::strcmp(line,"the rest of this line")==0);std::fclose(file);
    file=std::tmpfile();assert(file);std::fputs("too-long",file);std::rewind(file);
    char small[3]={'x','x','x'};assert(fscanf_s(file,"%s",small,sizeof(small))==0&&small[0]==0&&errno==ERANGE);std::fclose(file);
    assert(fopen_s(nullptr,"x","r")==EINVAL);FILE* missing=reinterpret_cast<FILE*>(1);assert(fopen_s(&missing,nullptr,"r")==EINVAL&&!missing);
    assert(strcpy_s(text,nullptr)==EINVAL&&text[0]==0);assert(_wstat64(nullptr,&stat)==-1&&errno==EINVAL);
    file=std::tmpfile();assert(file);assert(fscanf_s(file,"%[unterminated",text,sizeof(text))==EOF&&errno==EINVAL);assert(fscanf_s(file,"%s",text,size_t(-1))==EOF&&errno==EINVAL);std::fclose(file);
    void* aligned=_aligned_malloc(37,64);assert(aligned&&((uintptr_t)aligned&63)==0);std::memset(aligned,0xa7,37);_aligned_free(aligned);
    auto* transient=_malloca(19);assert(transient);std::memset(transient,0xa3,19);_freea(transient);
    assert(_wtof(L" 1.25 tail")==1.25);
    std::puts("CRT: wide paths, file metadata, secure fields and alignment passed");
}
