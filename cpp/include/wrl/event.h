#pragma once
#include "Windows.h"
namespace Microsoft { namespace WRL { namespace Wrappers {
class FileHandle {
    HANDLE value;
public:
    explicit FileHandle(HANDLE h):value(h){}
    ~FileHandle(){if(value && value!=INVALID_HANDLE_VALUE)CloseHandle(value);}
    FileHandle(const FileHandle&)=delete;
    FileHandle& operator=(const FileHandle&)=delete;
    HANDLE Get()const{return value;}
};
}}}
