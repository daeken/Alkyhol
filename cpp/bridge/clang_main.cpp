#include "llvm/Support/LLVMDriver.h"
extern int clang_main(int,char**,const llvm::ToolContext&);
int main(int argc,char**argv){return clang_main(argc,argv,{argv[0],nullptr,false});}
