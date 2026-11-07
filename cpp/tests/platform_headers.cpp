// Copyright (c) Alkyhol contributors. MIT licensed.
#include "Windows.h"
#include "ppltasks.h"
#include <cassert>
#include <cstdint>
#include <future>
#include <stdexcept>
int main(){
    static_assert(sizeof(DWORD)==4 && sizeof(LONG)==4 && sizeof(HRESULT)==4,"Windows scalar widths");
    static_assert(BOOL(-1)<0,"BOOL signedness");
    static_assert(sizeof(GUID)==16 && sizeof(HWND)==sizeof(void*) && sizeof(LARGE_INTEGER)==8,"host COM/handle layouts");
    LARGE_INTEGER value{};value.QuadPart=-2;assert(value.LowPart==0xfffffffe && value.HighPart==-1);
    int array[7]={};static_assert(_countof(array)==7,"array length");
    std::promise<void> go;
    auto future=go.get_future();
    auto task=concurrency::create_task([f=std::move(future)]()mutable{f.wait();return 42;});
    go.set_value();task.wait();assert(task.get()==42);assert(task.get()==42);
    auto bad=concurrency::create_task([]()->int{throw std::runtime_error("expected");});
    bool observed=false;try{bad.get();}catch(const std::runtime_error&){observed=true;}assert(observed);
    auto nothing=concurrency::create_task([]{});nothing.get();
}
