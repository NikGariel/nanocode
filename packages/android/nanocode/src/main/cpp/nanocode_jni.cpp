#include <jni.h>
#include <vector>
#include <cstdint>
#include "nanocode.h"
static void fail(JNIEnv* e, const char* message) { e->ThrowNew(e->FindClass("java/lang/IllegalArgumentException"), message); }
static std::vector<uint8_t> copy(JNIEnv* e, jbyteArray a) {
    std::vector<uint8_t> v(static_cast<size_t>(e->GetArrayLength(a)));
    if (!v.empty()) e->GetByteArrayRegion(a, 0, static_cast<jsize>(v.size()), reinterpret_cast<jbyte*>(v.data()));
    return v;
}
extern "C" JNIEXPORT jstring JNICALL Java_dev_nanocode_NanoCode_svg(JNIEnv* e, jclass, jbyteArray a, jint id, jint one, jint zero, jint bg) {
    if (!a || e->GetArrayLength(a)>40) { fail(e,"Payload must contain 0..40 bytes"); return nullptr; }
    auto v=copy(e,a);
    int n=nc_svg_ex(v.data(),v.size(),id,one,zero,bg,nullptr,0);
    if(n<0){fail(e,"Invalid format, capacity or palette");return nullptr;}
    std::vector<uint8_t> out(static_cast<size_t>(n)+1,0);
    int written=nc_svg_ex(v.data(),v.size(),id,one,zero,bg,out.data(),n);
    if(written!=n){fail(e,"SVG generation failed");return nullptr;}
    return e->NewStringUTF(reinterpret_cast<const char*>(out.data())); // SVG output is ASCII.
}
extern "C" JNIEXPORT jobject JNICALL Java_dev_nanocode_NanoCode_scan(JNIEnv* e,jclass,jbyteArray a,jint w,jint h) {
    if(!a||w<32||h<32||w>2048||h>2048||e->GetArrayLength(a)!=w*h){fail(e,"Expected packed grayscale, 32..2048 per dimension");return nullptr;}
    auto gray=copy(e,a);uint8_t out[40];double corners[8];uint32_t metadata[2];
    int n=nc_scan_ex(gray.data(),w,h,out,40,corners,metadata);
    if(n==-3)return nullptr;
    if(n<0){fail(e,"Native scan failed");return nullptr;}
    jbyteArray bytes=e->NewByteArray(n);if(!bytes)return nullptr;
    e->SetByteArrayRegion(bytes,0,n,reinterpret_cast<jbyte*>(out));
    jdoubleArray points=e->NewDoubleArray(8);if(!points)return nullptr;
    e->SetDoubleArrayRegion(points,0,8,corners);
    jclass c=e->FindClass("dev/nanocode/NanoCode$Detection");if(!c)return nullptr;
    jmethodID init=e->GetMethodID(c,"<init>","([BII[D)V");if(!init)return nullptr;
    return e->NewObject(c,init,bytes,metadata[0],metadata[1],points);
}
extern "C" JNIEXPORT jint JNICALL Java_dev_nanocode_NanoCode_paletteScore(JNIEnv*,jclass,jint one,jint zero,jint bg){return nc_palette_score(one,zero,bg);}
extern "C" JNIEXPORT jintArray JNICALL Java_dev_nanocode_NanoCode_backgrounds(JNIEnv* e,jclass,jint one,jint zero){
    uint32_t colors[8];int n=nc_backgrounds(one,zero,colors,8);
    if(n<0){fail(e,"Invalid palette");return nullptr;}
    jintArray a=e->NewIntArray(n);if(!a)return nullptr;
    e->SetIntArrayRegion(a,0,n,reinterpret_cast<jint*>(colors));return a;
}
