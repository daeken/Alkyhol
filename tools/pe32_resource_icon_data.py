"""Original MIT owned icon pattern producer; never reads game assets."""
import struct
def pixels(size):
    return bytes(v for y in range(size) for x in range(size) for v in [(x*7)%256,(y*5)%256,((x^y)*11)%256,255])
def dib(size):
    color=pixels(size);stride=size*4
    rows=b''.join(color[y*stride:(y+1)*stride] for y in reversed(range(size)))
    return struct.pack('<IiiHHIIiiII',40,size,size*2,1,32,0,len(color),0,0,0,0)+rows+bytes(((size+31)//32)*4*size)
def resources():
    small,large=dib(16),dib(32)
    group=struct.pack('<HHH',0,1,2)+struct.pack('<BBBBHHIH',16,16,0,0,1,32,len(small),2)+struct.pack('<BBBBHHIH',32,32,0,0,1,32,len(large),1)
    return [(3,1,1033,large),(3,2,1033,small),(14,41,1033,group)]
