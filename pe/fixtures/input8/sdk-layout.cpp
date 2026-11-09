// Independent Windows SDK checks; never includes Alkyhol headers.
#define DIRECTINPUT_VERSION 0x0800
#define CINTERFACE
#include <Windows.h>
#include <dinput.h>
#include <cstddef>
static_assert(sizeof(void*)==4);
static_assert(sizeof(DIDEVCAPS)==44);
static_assert(sizeof(DIDEVICEINSTANCEA)==580);
static_assert(sizeof(DIDEVICEINSTANCEW)==1100);
static_assert(sizeof(DIDEVICEOBJECTINSTANCEA)==316);
static_assert(sizeof(DIDEVICEOBJECTINSTANCEW)==576);
static_assert(sizeof(DIDATAFORMAT)==24);
static_assert(sizeof(DIOBJECTDATAFORMAT)==16);
static_assert(sizeof(DIPROPDWORD)==20);
static_assert(sizeof(DIPROPPOINTER)==20);
static_assert(sizeof(DIDEVICEOBJECTDATA)==20);
static_assert(offsetof(DIDEVICEINSTANCEW,tszInstanceName)==40);
static_assert(offsetof(DIDEVICEOBJECTINSTANCEW,dwOfs)==20);
static_assert(offsetof(IDirectInput8AVtbl,CreateDevice)==3*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,GetCapabilities)==3*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,GetProperty)==5*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,SetProperty)==6*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,Acquire)==7*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,GetDeviceState)==9*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,GetDeviceData)==10*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,SetDataFormat)==11*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,SetCooperativeLevel)==13*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,GetObjectInfo)==14*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,GetDeviceInfo)==15*4);
static_assert(offsetof(IDirectInputDevice8AVtbl,Poll)==25*4);
