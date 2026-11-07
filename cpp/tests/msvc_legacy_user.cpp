// Copyright (c) Alkyhol contributors. MIT licensed.
#include "msvc_legacy.h"
#include <cassert>
#include <type_traits>
static_assert(std::is_same<LegacyOwner,std::unique_ptr<int>>::value,"typedef preserves identity");
int main(){LegacyOwner value(new int(13));assert(legacy_transform(*value)==42);}
