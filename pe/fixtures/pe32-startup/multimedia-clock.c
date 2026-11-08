/* Original MIT variant for DAO's actually reached Winmm boot clock.
 * The existing counter/guard/atomic producer is unchanged; only the called
 * millisecond API is selected here, from a genuine separate Winmm import. */
#define GetTickCount timeGetTime
#include "startup-clock.c"
