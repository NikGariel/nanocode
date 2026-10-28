#include "nanocode.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(void) {
    uint8_t data[40], decoded[40];
    memset(data, 'A', sizeof data);
    uint8_t *image=malloc(512*512);
    if (!image) return 1;
    for (uint32_t id=1; id<=4; ++id) {
        size_t n=(size_t)nc_format_capacity(id);
        uint32_t metadata[2]={0,0};
        if(nc_render_ex(data,n,id,512,0xdddddd,0x777777,0x101010,image,512*512)<0) return 2;
        int result=nc_scan_ex(image,512,512,decoded,sizeof decoded,NULL,metadata);
        if(result!=(int)n || metadata[0]!=id || memcmp(data,decoded,n)) return 3;
        printf("C ABI: format %u, %zu bytes, %d rings\n",id,n,nc_format_rings(id));
    }
    free(image);
    return 0;
}
