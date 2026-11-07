// Copyright (c) Alkyhol contributors. MIT licensed.
#include <cassert>
#include <cstdint>
static int destroyed=0;
struct Padding {std::uint64_t padding[3]={1,2,3};virtual ~Padding()=default;};
struct Base {virtual void increment()=0;virtual int read()const=0;virtual ~Base()=default;};
struct Derived:Padding,Base {int value=37;void increment()override{++value;}int read()const override{return value;}~Derived(){++destroyed;}};
int invoke(Base& base){assert(destroyed==0);base.increment();return base.read();}
int main(){int result=invoke(Derived{});assert(result==38);assert(destroyed==1);}
