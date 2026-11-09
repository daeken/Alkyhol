/* Alkyhol MIT: recvmsg's SDK layout and truncation flag stay on the C side. */
#include <sys/socket.h>
#include <sys/uio.h>
#include <stdint.h>
#include <errno.h>
int64_t alkyhol_udp_receive(int fd,void* data,uint32_t capacity,void* address,uint32_t* address_length,int32_t* truncated,int32_t* error) {
    struct iovec vector={data,capacity};
    struct msghdr message={0};
    message.msg_name=address;message.msg_namelen=*address_length;
    message.msg_iov=&vector;message.msg_iovlen=1;
    ssize_t result=recvmsg(fd,&message,0);
    if(result<0){*error=errno;return -1;}
    *address_length=message.msg_namelen;*truncated=(message.msg_flags&MSG_TRUNC)!=0;*error=0;
    return result;
}
