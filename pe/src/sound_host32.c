/* Alkyhol MIT. Public AudioQueue output, with synchronous teardown. */
#include <AudioToolbox/AudioToolbox.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdatomic.h>
typedef void (*fill_fn)(void*,float*,uint32_t,uint32_t);
struct output {AudioQueueRef queue; fill_fn fill; void *user; atomic_int status;};
static void refill(void *user,AudioQueueRef queue,AudioQueueBufferRef buffer){
    struct output *out=user;
    out->fill(out->user,buffer->mAudioData,512,1);
    buffer->mAudioDataByteSize=512*2*sizeof(float);
    OSStatus status=AudioQueueEnqueueBuffer(queue,buffer,0,NULL);
    if(status)atomic_store(&out->status,status);
}
int32_t alkyhol_sound_open(void *user,fill_fn fill,void **result){
    *result=NULL;struct output *out=calloc(1,sizeof(*out));if(!out)return -108;
    out->user=user;out->fill=fill;atomic_init(&out->status,0);
    AudioStreamBasicDescription format={0};format.mSampleRate=48000;
    format.mFormatID=kAudioFormatLinearPCM;format.mFormatFlags=kAudioFormatFlagIsFloat|kAudioFormatFlagIsPacked;
    format.mBytesPerPacket=8;format.mFramesPerPacket=1;format.mBytesPerFrame=8;format.mChannelsPerFrame=2;format.mBitsPerChannel=32;
    OSStatus status=AudioQueueNewOutput(&format,refill,out,NULL,NULL,0,&out->queue);
    if(status){free(out);return status;}
    for(int i=0;i<4;++i){
        AudioQueueBufferRef buffer=NULL;status=AudioQueueAllocateBuffer(out->queue,512*8,&buffer);if(status)break;
        fill(user,buffer->mAudioData,512,0);buffer->mAudioDataByteSize=512*8;
        status=AudioQueueEnqueueBuffer(out->queue,buffer,0,NULL);if(status)break;
    }
    if(!status)status=AudioQueueStart(out->queue,NULL);
    if(status){AudioQueueDispose(out->queue,1);free(out);return status;}
    *result=out;return 0;
}
void alkyhol_sound_close(void *pointer){
    struct output *out=pointer;if(!out)return;
    AudioQueueStop(out->queue,1);AudioQueueDispose(out->queue,1);free(out);
}

int32_t alkyhol_sound_status(void *pointer){struct output *out=pointer;return atomic_load(&out->status);}
