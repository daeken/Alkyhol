// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
#include <codecvt>
#include <locale>
#include <string>
#include <algorithm>
inline std::string alkyhol_path(const wchar_t* path) {
    auto text=std::wstring_convert<std::codecvt_utf8<wchar_t>>{}.to_bytes(path);
    std::replace(text.begin(),text.end(),'\\','/');return text;
}
inline std::string alkyhol_path(const std::wstring& path){return alkyhol_path(path.c_str());}
