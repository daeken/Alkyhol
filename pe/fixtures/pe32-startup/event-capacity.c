/* Original MIT test for DAO's live event/semaphore allocation and close lifecycle.
 * Windows and Alkyhol execute the identical PE; no game-derived inputs. */
typedef unsigned int U;typedef void *H;
#define API __declspec(dllimport)
API H __stdcall GetStdHandle(U);API int __stdcall WriteFile(H,const void *,U,U *,H);
API H __stdcall CreateEventW(H,int,int,const unsigned short *);
API H __stdcall CreateEventA(H,int,int,const char *);
API H __stdcall CreateSemaphoreW(H,int,int,const unsigned short *);
API int __stdcall SetEvent(H);API int __stdcall ResetEvent(H);
API U __stdcall WaitForSingleObjectEx(H,U,int);
API int __stdcall ReleaseSemaphore(H,int,int *);API int __stdcall CloseHandle(H);
static H events[512],semaphores[512];static U words[24],used;
static U stack(void){U n;__asm__ volatile("movl %%esp,%0":"=r"(n));return n;}
static void put(U n){if(used>=24)__builtin_trap();words[used++]=n;}
static U wait(H h){return WaitForSingleObjectEx(h,0,0);}
static U distinct(void){
 for(U i=0;i<512;i++){
  if(!events[i]||!semaphores[i]||events[i]==semaphores[i])return 0;
  for(U j=i+1;j<512;j++)if(events[i]==events[j]||semaphores[i]==semaphores[j])return 0;
  for(U j=0;j<512;j++)if(events[i]==semaphores[j])return 0;
 }return 1;
}
U entry(void){
 U before=stack(),ok=1;put(0x45564350);put(1);
 for(U i=0;i<512;i++){
  events[i]=i&1?CreateEventW(0,i&2,1,0):CreateEventA(0,i&2,1,0);
  semaphores[i]=CreateSemaphoreW(0,1,3,0);
  if(!events[i]||!semaphores[i])ok=0;
 }put(ok);put(distinct());
 ok=1;for(U i=0;i<512;i++)if(wait(events[i])!=0||wait(events[i])!=((i&2)?0u:258u))ok=0;put(ok);
 ok=1;for(U i=0;i<512;i++){
  if(!ResetEvent(events[i])||wait(events[i])!=258||!SetEvent(events[i])||wait(events[i])!=0||wait(events[i])!=((i&2)?0u:258u))ok=0;
 }put(ok);
 ok=1;for(U i=0;i<512;i++){
  int previous=-1;
  if(wait(semaphores[i])!=0||wait(semaphores[i])!=258||!ReleaseSemaphore(semaphores[i],2,&previous)||previous!=0||wait(semaphores[i])!=0||wait(semaphores[i])!=0||wait(semaphores[i])!=258)ok=0;
 }put(ok);
 ok=1;for(U i=0;i<512;i+=2){if(!CloseHandle(events[i])||!CloseHandle(semaphores[i]))ok=0;events[i]=semaphores[i]=0;}put(ok);
 ok=1;for(U i=0;i<512;i+=2){events[i]=CreateEventA(0,0,0,0);semaphores[i]=CreateSemaphoreW(0,3,3,0);if(!events[i]||!semaphores[i])ok=0;}put(ok);put(distinct());
 ok=1;for(U i=0;i<512;i++){
  if(wait(events[i])!=((i&1)&&(i&2)?0u:258u))ok=0;
  for(U n=0;n<3;n++)if(wait(semaphores[i])!=((i&1)?258u:0u))ok=0;
  if(wait(semaphores[i])!=258)ok=0;
 }put(ok);
 ok=1;for(U i=0;i<512;i++)if(!CloseHandle(events[i])||!CloseHandle(semaphores[i]))ok=0;put(ok);
 put(before==stack());put(used+1);U done=0;
 if(!WriteFile(GetStdHandle((U)-11),words,used*4,&done,0)||done!=used*4)__builtin_trap();return 0;
}
