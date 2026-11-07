// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
#include <cstdlib>
#include <cerrno>
#include <alloca.h>
#define _alloca alloca
inline void* _aligned_malloc(size_t size,size_t alignment){
    if(!size||!alignment||(alignment&(alignment-1))){errno=EINVAL;return nullptr;}
    if(alignment<sizeof(void*))alignment=sizeof(void*);
    void* data=nullptr;int code=posix_memalign(&data,alignment,size);if(code){errno=code;return nullptr;}return data;
}
inline void _aligned_free(void* data){std::free(data);}
inline void* _malloca(size_t size){return std::malloc(size);}
inline void _freea(void* data){std::free(data);}
