/* Alkyhol MIT. Enumerate actual CoreAudio devices with input/output streams.
 * No synthetic default device or cached Windows hardware identity. */
#include <CoreAudio/CoreAudio.h>
#include <stdint.h>
#include <stdlib.h>
int32_t alkyhol_audio_devices(uint32_t input,uint32_t* devices,uint32_t capacity,uint32_t* count) {
    AudioObjectPropertyAddress address={kAudioHardwarePropertyDevices,kAudioObjectPropertyScopeGlobal,kAudioObjectPropertyElementMain};
    UInt32 bytes=0;OSStatus status=AudioObjectGetPropertyDataSize(kAudioObjectSystemObject,&address,0,NULL,&bytes);
    if(status)return status;
    if(bytes%sizeof(AudioDeviceID)||bytes/sizeof(AudioDeviceID)>capacity)return -50;
    AudioDeviceID* all=malloc(bytes?bytes:1);if(!all)return -108;
    status=AudioObjectGetPropertyData(kAudioObjectSystemObject,&address,0,NULL,&bytes,all);
    if(status){free(all);return status;}
    uint32_t used=0;
    address.mSelector=kAudioDevicePropertyStreams;address.mScope=input?kAudioObjectPropertyScopeInput:kAudioObjectPropertyScopeOutput;
    for(UInt32 i=0;i<bytes/sizeof(AudioDeviceID);i++){
        UInt32 streams=0;status=AudioObjectGetPropertyDataSize(all[i],&address,0,NULL,&streams);
        if(status){free(all);return status;}
        if(streams){if(used==capacity){free(all);return -50;}devices[used++]=all[i];}
    }
    free(all);*count=used;return 0;
}
