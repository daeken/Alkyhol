/* Original MIT: share the exact stack/state witness while changing the operand
 * operation. The pinned sqrt producer is never edited for a new intrinsic. */
#define _run_sqrt _run_atan
#define __imp___CIsqrt __imp___CIatan
#include "ci-sqrt.S"
