// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
#include "alkyhol_path.h"
#include <sys/stat.h>
#include <cerrno>
#include <cstdio>
#include <cstring>
#include <cwchar>
#include <cstdlib>
#include <cstdint>
#include <vector>
#include <limits>
#include <algorithm>
#undef st_atime
#undef st_mtime
#undef st_ctime
struct _stat64 {
    uint32_t st_dev;uint16_t st_ino,st_mode;int16_t st_nlink,st_uid,st_gid;
    uint32_t st_rdev;int64_t st_size,st_atime,st_mtime,st_ctime;
};
inline int _wstat64(const wchar_t* path,_stat64* out) {
    if(!path||!out){errno=EINVAL;return -1;}
    struct stat native{};
    std::string filename;
    try{filename=alkyhol_path(path);}catch(const std::range_error&){errno=EILSEQ;return -1;}
    if(::stat(filename.c_str(),&native)!=0)return -1;
    *out={};out->st_dev=uint32_t(native.st_dev);out->st_ino=uint16_t(native.st_ino);
    out->st_mode=uint16_t(native.st_mode);out->st_nlink=int16_t(native.st_nlink);
    out->st_uid=int16_t(native.st_uid);out->st_gid=int16_t(native.st_gid);out->st_rdev=uint32_t(native.st_rdev);
    out->st_size=native.st_size;
#ifdef __APPLE__
    out->st_atime=native.st_atimespec.tv_sec;out->st_mtime=native.st_mtimespec.tv_sec;out->st_ctime=native.st_ctimespec.tv_sec;
#else
    out->st_atime=native.st_atim.tv_sec;out->st_mtime=native.st_mtim.tv_sec;out->st_ctime=native.st_ctim.tv_sec;
#endif
    return 0;
}
inline double _wtof(const wchar_t* text){return std::wcstod(text,nullptr);}
inline int fopen_s(FILE** out,const char* path,const char* mode) {
    if(!out)return EINVAL;*out=nullptr;if(!path||!mode)return EINVAL;
    *out=std::fopen(path,mode);return *out?0:errno;
}
inline int strcpy_s(char* out,size_t capacity,const char* source) {
    if(!out||!capacity)return EINVAL;
    if(!source){out[0]=0;return EINVAL;}
    size_t length=std::strlen(source);
    if(length>=capacity){out[0]=0;return ERANGE;}
    std::memcpy(out,source,length+1);return 0;
}
template<size_t N> inline int strcpy_s(char(&out)[N],const char* source){return strcpy_s(out,N,source);}
template<class T> inline int fscanf_s(FILE* file,const char* format,T* output){return std::fscanf(file,format,output);}
// MiniEngine uses one unsuppressed narrow string conversion in these overloads.
// Bound a temporary read to the supplied capacity, then reject a field which
// cannot fit its terminator; never write a truncated successful result.
template<class Size> inline int fscanf_s(FILE* file,const char* format,char* output,Size capacity) {
    if(!file||!format||!output||capacity<=0){errno=EINVAL;return EOF;}
    size_t size=size_t(capacity),conversion=0;
    if(size==std::numeric_limits<size_t>::max()){errno=EINVAL;return EOF;}
    std::string bounded=format;
    for(size_t i=0;i<bounded.size();++i) {
        if(bounded[i]!='%')continue;
        ++i;if(i<bounded.size()&&bounded[i]=='%')continue;
        bool suppressed=i<bounded.size()&&bounded[i]=='*';if(suppressed)++i;
        size_t digits=i;while(i<bounded.size()&&bounded[i]>='0'&&bounded[i]<='9')++i;
        size_t end=i;
        if(i>=bounded.size()){errno=EINVAL;return EOF;}
        char type=bounded[i];
        if(type=='['){++i;if(i<bounded.size()&&bounded[i]=='^')++i;if(i<bounded.size()&&bounded[i]==']')++i;while(i<bounded.size()&&bounded[i]!=']')++i;if(i==bounded.size()){errno=EINVAL;return EOF;}}
        if(suppressed)continue;
        if(++conversion>1||(type!='s'&&type!='[')){errno=ENOSYS;return EOF;}
        size_t width=size;
        if(end>digits)width=std::min(width,size_t(std::strtoull(bounded.substr(digits,end-digits).c_str(),nullptr,10)));
        if(width==0){errno=EINVAL;return EOF;}
        std::string number=std::to_string(width);
        bounded.replace(digits,end-digits,number);i=i-(end-digits)+number.size();
    }
    if(conversion!=1){errno=ENOSYS;return EOF;}
    std::vector<char> temporary(size+1,0);int result=std::fscanf(file,bounded.c_str(),temporary.data());
    if(result==1){size_t length=std::strlen(temporary.data());if(length>=size){output[0]=0;errno=ERANGE;return 0;}std::memcpy(output,temporary.data(),length+1);}
    return result;
}
#include <strings.h>
#define _stricmp strcasecmp
#define _wcsicmp wcscasecmp
