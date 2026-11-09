// Original MIT independent SDK layouts and actual named/ordinal export identity.
#include <Winsock2.h>
#include <Windows.h>
#include <cstddef>
static_assert(sizeof(void*)==4);
static_assert(sizeof(WSADATA)==400);
static_assert(offsetof(WSADATA,szDescription)==4);
static_assert(offsetof(WSADATA,szSystemStatus)==261);
static_assert(offsetof(WSADATA,iMaxSockets)==390);
static_assert(offsetof(WSADATA,lpVendorInfo)==396);
static_assert(sizeof(hostent)==16);
static_assert(offsetof(hostent,h_addr_list)==12);
static_assert(sizeof(sockaddr_in)==16);
static_assert(sizeof(OSVERSIONINFOEXA)==156);
static_assert(offsetof(OSVERSIONINFOEXA,wProductType)==154);
static_assert(sizeof(OSVERSIONINFOEXW)==284);
struct Export {const char *name;WORD ordinal;};
int main(){const auto module=LoadLibraryW(L"ws2_32.dll");if(!module)return 1;const Export names[]={{"WSAStartup",115},{"WSACleanup",116},{"WSAGetLastError",111},{"socket",23},{"closesocket",3},{"connect",4},{"htons",9},{"inet_addr",11},{"ioctlsocket",10},{"recv",16},{"send",19},{"gethostbyname",52},{"gethostbyaddr",51}};for(const auto &entry:names){const auto pointer=GetProcAddress(module,entry.name);if(!pointer||pointer!=GetProcAddress(module,MAKEINTRESOURCEA(entry.ordinal)))return 2;}return FreeLibrary(module)?0:3;}
