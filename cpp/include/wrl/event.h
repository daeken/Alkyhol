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
#include "roapi.h"
namespace Microsoft { namespace WRL { namespace Wrappers {
class RoInitializeWrapper {
    HRESULT result;
public:
    explicit RoInitializeWrapper(RO_INIT_TYPE type):result(RoInitialize(type)){}
    ~RoInitializeWrapper(){if(SUCCEEDED(result))RoUninitialize();}
    operator HRESULT()const{return result;}
    RoInitializeWrapper(const RoInitializeWrapper&)=delete;
    RoInitializeWrapper& operator=(const RoInitializeWrapper&)=delete;
};
}}}
