// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
#include <cwchar>
#include <cstdarg>
#include <string>
#include <cerrno>
// Translate the Windows wide-printf string/character conventions without
// changing arguments. This is a bounded source ABI adapter, not a UCRT clone.
inline bool alkyhol_wide_format(const wchar_t* format,std::wstring& out){
    if(!format){errno=EINVAL;return false;}
    out.clear();
    for(size_t i=0;format[i];){
        if(format[i]!=L'%'){out+=format[i++];continue;}
        out+=format[i++];if(format[i]==L'%'){out+=format[i++];continue;}
        while(format[i]&&std::wcschr(L"-+ #0",format[i]))out+=format[i++];
        if(format[i]==L'*')out+=format[i++];else while(format[i]>=L'0'&&format[i]<=L'9')out+=format[i++];
        if(format[i]==L'.'){out+=format[i++];if(format[i]==L'*')out+=format[i++];else while(format[i]>=L'0'&&format[i]<=L'9')out+=format[i++];}
        std::wstring length;
        if(format[i]&&std::wcschr(L"hlwztjL",format[i])){length+=format[i++];if((length==L"h"||length==L"l")&&format[i]==length[0])length+=format[i++];}
        if(!format[i]){errno=EINVAL;return false;}
        wchar_t type=format[i++];
        if(type==L's'||type==L'S'||type==L'c'||type==L'C'){
            bool wide=length==L"l"||length==L"w"||(length.empty()&&(type==L's'||type==L'c'));
            if(!length.empty()&&length!=L"h"&&length!=L"l"&&length!=L"w"){errno=ENOSYS;return false;}
            if(wide)out+=L'l';out+=(type==L's'||type==L'S')?L's':L'c';
        }else{
            if(!std::wcschr(L"diouxXfFeEgGaAp",type)||length==L"w"){errno=ENOSYS;return false;}
            out+=length;out+=type;
        }
    }
    return true;
}
inline int alkyhol_vswprintf(wchar_t* out,size_t capacity,const wchar_t* format,va_list args){
    if(!out||!capacity){errno=EINVAL;return -1;}
    std::wstring native;if(!alkyhol_wide_format(format,native)){out[0]=0;return -1;}
    int result=std::vswprintf(out,capacity,native.c_str(),args);if(result<0)out[0]=0;return result;
}
inline int alkyhol_wprintf(const wchar_t* format,...){
    std::wstring native;if(!alkyhol_wide_format(format,native))return -1;
    va_list args;va_start(args,format);int result=std::vwprintf(native.c_str(),args);va_end(args);return result;
}
inline int alkyhol_swprintf(wchar_t* out,size_t capacity,const wchar_t* format,...){
    va_list args;va_start(args,format);int result=alkyhol_vswprintf(out,capacity,format,args);va_end(args);return result;
}
#define vswprintf alkyhol_vswprintf
#define wprintf alkyhol_wprintf
