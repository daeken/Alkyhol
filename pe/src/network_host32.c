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

/* Windows arrays/timevals are translated into actual SDK structures here. */
#include <sys/select.h>
int32_t alkyhol_socket_select(const int32_t* fds,const uint32_t* counts,int32_t seconds,int32_t microseconds,int32_t finite,uint8_t* selected,int32_t* error) {
    fd_set sets[3];int maximum=-1;uint32_t position=0;
    for(uint32_t group=0;group<3;group++){
        FD_ZERO(&sets[group]);
        for(uint32_t i=0;i<counts[group];i++){
            int fd=fds[position++];
            if(fd<0){*error=EBADF;return -1;}
            if(fd>=FD_SETSIZE){*error=EOVERFLOW;return -1;}
            FD_SET(fd,&sets[group]);if(fd>maximum)maximum=fd;
        }
    }
    struct timeval timeout={seconds,microseconds};
    int result=select(maximum+1,counts[0]?&sets[0]:0,counts[1]?&sets[1]:0,counts[2]?&sets[2]:0,finite?&timeout:0);
    if(result<0){*error=errno;return -1;}
    position=0;
    for(uint32_t group=0;group<3;group++){
        for(uint32_t i=0;i<counts[group];i++){
            selected[position]=FD_ISSET(fds[position],&sets[group])!=0;position++;
        }
    }
    *error=0;return result;
}
