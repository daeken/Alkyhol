/* Original MIT public-API glue for ScreenCaptureKit's Objective-C blocks.
 * No guest pointers or game metadata enter this adapter. UI coordinates use
 * the explicitly selected logical desktop dimensions supplied by Rust. */
#import <ScreenCaptureKit/ScreenCaptureKit.h>
#import <CoreGraphics/CoreGraphics.h>
#import <CoreFoundation/CoreFoundation.h>
#include <stdint.h>
static bool await_capture(dispatch_semaphore_t semaphore) {
    CFAbsoluteTime end=CFAbsoluteTimeGetCurrent()+5.0;
    while(dispatch_semaphore_wait(semaphore,DISPATCH_TIME_NOW)!=0) {
        if(CFAbsoluteTimeGetCurrent()>=end)return false;
        CFRunLoopRunInMode(kCFRunLoopDefaultMode,0.005,true);
    }
    return true;
}
static int draw_image(CGImageRef image,size_t width,size_t height,uint8_t *bytes) {
    CGColorSpaceRef color=CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    if(!color)return 31;
    CGContextRef context=CGBitmapContextCreate(bytes,width,height,8,width*4,color,kCGImageAlphaPremultipliedLast|kCGBitmapByteOrder32Big);
    CGColorSpaceRelease(color);if(!context)return 31;
    CGContextDrawImage(context,CGRectMake(0,0,width,height),image);CGContextRelease(context);
    for(size_t i=0;i<width*height;++i){uint8_t red=bytes[i*4];bytes[i*4]=bytes[i*4+2];bytes[i*4+2]=red;bytes[i*4+3]=255;}
    return 0;
}
int alkyhol_pe32_capture_display(size_t width,size_t height,uint8_t *bytes,size_t length) {
    if(!width||!height||width>8192||height>8192||length!=width*height*4||!bytes)return 87;
    if(!CGPreflightScreenCaptureAccess())return 5;
    if(@available(macOS 14.0,*)) {@autoreleasepool {
        dispatch_semaphore_t semaphore=dispatch_semaphore_create(0);
        __block SCShareableContent *content=nil;__block NSError *error=nil;
        [SCShareableContent getShareableContentExcludingDesktopWindows:NO onScreenWindowsOnly:YES completionHandler:^(SCShareableContent *value,NSError *failure){content=value;error=failure;dispatch_semaphore_signal(semaphore);}];
        if(!await_capture(semaphore))return 1460;if(error||!content)return 31;
        SCDisplay *display=nil;CGDirectDisplayID primary=CGMainDisplayID();
        for(SCDisplay *candidate in content.displays)if(candidate.displayID==primary){display=candidate;break;}
        if(!display)return 31;
        SCContentFilter *filter=[[SCContentFilter alloc]initWithDisplay:display excludingWindows:@[]];
        SCStreamConfiguration *config=[[SCStreamConfiguration alloc]init];config.width=width;config.height=height;config.showsCursor=NO;
        __block CGImageRef image=NULL;
        [SCScreenshotManager captureImageWithFilter:filter configuration:config completionHandler:^(CGImageRef value,NSError *failure){if(value)image=CGImageRetain(value);error=failure;dispatch_semaphore_signal(semaphore);}];
        if(!await_capture(semaphore))return 1460;
        if(error||!image){if(image)CGImageRelease(image);return 31;}
        int status=draw_image(image,width,height,bytes);CGImageRelease(image);return status;
    }}
    return 50;
}
int alkyhol_pe32_capture_color_contract(void) {
    uint8_t source[]={1,2,3,255,4,5,6,255,7,8,9,255,10,11,12,255},out[16]={0};
    CGColorSpaceRef color=CGColorSpaceCreateWithName(kCGColorSpaceSRGB);if(!color)return 0;
    CGContextRef context=CGBitmapContextCreate(source,2,2,8,8,color,kCGImageAlphaPremultipliedLast|kCGBitmapByteOrder32Big);CGColorSpaceRelease(color);if(!context)return 0;
    CGImageRef image=CGBitmapContextCreateImage(context);CGContextRelease(context);if(!image)return 0;
    int status=draw_image(image,2,2,out);CGImageRelease(image);if(status)return 0;
    for(size_t i=0;i<4;++i)if(out[i*4]!=source[i*4+2]||out[i*4+1]!=source[i*4+1]||out[i*4+2]!=source[i*4]||out[i*4+3]!=255)return 0;
    return 1;
}
