#include "nanocode.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(void) {
    const uint8_t text[] = "00NanoCode";
    const size_t size = 512;
    uint8_t *pixels = malloc(size * size), decoded[24];
    double corners[8];
    if (!pixels || nc_render(text, sizeof(text)-1,size,pixels,size*size)<0) return 1;
    int n=nc_scan(pixels,size,size,decoded,sizeof(decoded),corners);
    free(pixels);
    if(n!=(int)sizeof(text)-1 || memcmp(text,decoded,(size_t)n)) return 2;
    printf("C ABI roundtrip: %.*s\n",n,decoded);
    return 0;
}
