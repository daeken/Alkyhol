// Copyright (c) Alkyhol contributors. MIT licensed.
#pragma once
#include <memory>
typedef public std::unique_ptr<int> LegacyOwner;
template<class T> T legacy_transform(T);
template int legacy_transform(int);
