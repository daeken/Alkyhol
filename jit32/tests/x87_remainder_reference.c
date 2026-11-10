/* MIT. Owned complete finite FPREM/FPREM1 observations; no game inputs. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif
#define XMM_CLOBBERS "xmm0","xmm1","xmm2","xmm3","xmm4","xmm5","xmm6","xmm7","xmm8","xmm9","xmm10","xmm11","xmm12","xmm13","xmm14","xmm15"
#define OBSERVE(op) __asm__ volatile("fxsave %[saved]; fxrstor %[in]; fnstenv %[before_env]; pushq %[initial]; popfq; " op "; fxsave %[out]; fnstenv %[env]; pushfq; popq %[flags]; fxrstor %[saved]" : [saved]"=m"(saved),[out]"=m"(out),[env]"=m"(env),[before_env]"=m"(before_env),[flags]"=&r"(flags) : [in]"m"(in),[initial]"r"(initial) : "st","st(1)","st(2)","st(3)","st(4)","st(5)","st(6)","st(7)","cc","memory",XMM_CLOBBERS)
typedef struct {uint64_t sig;uint16_t exponent;} Number;
static uint64_t random_state=UINT64_C(0xace013579bdf2468);
static uint64_t random_bits(void){random_state=random_state*UINT64_C(6364136223846793005)+UINT64_C(1442695040888963407);return random_state;}
static void operands(unsigned i,Number *x,Number *y){
 const uint64_t sig[]={UINT64_C(0x8000000000000000),UINT64_C(0x8000000000000001),UINT64_C(0x8000000000000002),UINT64_C(0xaaaaaaaaaaaaaaaa),UINT64_C(0xaaaaaaab00000000),UINT64_C(0xc000000000000000),UINT64_C(0xc000000000000001),UINT64_C(0xffffffffffffffff),UINT64_C(0x9000000000000000),UINT64_C(0xb000000000000000),UINT64_C(0xd000000000000000),UINT64_C(0xe000000000000000),UINT64_C(0x8888890000000000),UINT64_C(0x8888880000000000),UINT64_C(0xf000000000000000),UINT64_C(0xeeeeeeeeeeeeeeee)};
 const uint64_t tiny[]={0,1,2,3,4,7,8,UINT64_C(0x80000000),UINT64_C(0x7fffffffffffffff),UINT64_C(0x8000000000000000),UINT64_C(0x8000000000000001),UINT64_C(0x8000000000000002),UINT64_C(0xaaaaaaaaaaaaaaaa),UINT64_C(0xc000000000000000),UINT64_C(0xfffffffffffffffe),UINT64_C(0xffffffffffffffff)};
 const int differences[]={-65,-2,-1,0,1,2,31,62,63};
 if(i<256){x->sig=sig[i%16];y->sig=sig[(i/16)%16];y->exponent=0x3fff;x->exponent=(uint16_t)(0x3fff+differences[(i/4)%9]);}
 else if(i<384){x->sig=tiny[i%16];y->sig=tiny[1+(i/16)%15];x->exponent=(uint16_t)(x->sig>>63);y->exponent=(uint16_t)(y->sig>>63);}
 else{x->sig=random_bits()|UINT64_C(0x8000000000000000);y->sig=random_bits()|UINT64_C(0x8000000000000000);y->exponent=(uint16_t)(0x100+random_bits()%0x7d00);x->exponent=(uint16_t)((int)y->exponent+(int)(random_bits()%191)-127);}
 x->exponent|=(i&1)?0x8000:0;y->exponent|=(i&2)?0x8000:0;
}
int main(void){
#ifdef _WIN32
 if(_setmode(_fileno(stdout),_O_BINARY)<0)return 4;
#endif
 Number x[512],y[512];for(unsigned i=0;i<512;i++)operands(i,&x[i],&y[i]);
 for(unsigned kind=0;kind<2;kind++){
  for(unsigned mode=0;mode<24;mode++){
   for(unsigned i=0;i<512;i++){
    unsigned char in[512] __attribute__((aligned(16)))={0},out[512] __attribute__((aligned(16)))={0},saved[512] __attribute__((aligned(16))),env[32]={0},before_env[32]={0};
    const unsigned pcs[]={0,2,3};uint16_t cw=(uint16_t)(0x7f|(pcs[(mode%12)/4]<<8)|((mode%4)<<10)|((mode/12)<<12));uint16_t before=(uint16_t)(0x4700|((i&4)?0x23:0));uint32_t mxcsr=0x1fa0;uint64_t initial=0xad7,flags;
    memcpy(in,&cw,2);memcpy(in+2,&before,2);in[4]=7;memcpy(in+24,&mxcsr,4);memcpy(in+32,&x[i].sig,8);memcpy(in+40,&x[i].exponent,2);memcpy(in+48,&y[i].sig,8);memcpy(in+56,&y[i].exponent,2);
    const uint64_t sentinel=UINT64_C(0xa900000000000000);const uint16_t sentinel_exp=0x4005;memcpy(in+64,&sentinel,8);memcpy(in+72,&sentinel_exp,2);
    for(unsigned b=160;b<416;b++)in[b]=(unsigned char)(b*37+11);
    if(kind){OBSERVE("fprem1");}else{OBSERVE("fprem");}
    uint16_t sw;memcpy(&sw,out+2,2);
    if((sw&0x3c00)!=0||memcmp(out,in,2)||out[4]!=7||memcmp(out+24,in+24,4)||memcmp(out+48,in+48,10)||memcmp(out+64,in+64,10)||memcmp(out+160,in+160,256))return 5;
    unsigned char row[80]={0};uint32_t header[]={kind,cw};memcpy(row,header,8);memcpy(row+8,in+32,10);memcpy(row+18,in+48,10);memcpy(row+28,out+32,10);memcpy(row+38,out+2,2);memcpy(row+40,out,2);row[42]=out[4];row[43]=(unsigned char)mode;memcpy(row+44,out+24,4);memcpy(row+48,out+64,10);memcpy(row+58,&before,2);uint32_t f=(uint32_t)initial,g=(uint32_t)flags;memcpy(row+60,&f,4);memcpy(row+64,&g,4);memcpy(row+68,&i,4);memcpy(row+72,env+8,2);memcpy(row+74,before_env+8,2);memcpy(row+76,before_env,2);memcpy(row+78,before_env+4,2);
    if(fwrite(row,sizeof row,1,stdout)!=1)return 1;
   }
  }
 }
 return 0;
}
