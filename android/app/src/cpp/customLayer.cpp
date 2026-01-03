#include <jni.h>
#include <android/log.h>
#include "customLayer.hpp"


CustomLayerHostImpl hostImpl = CustomLayerHostImpl();

jlong getCustomLayer(JNIEnv*, jobject) {
    hostImpl.setup();
    return (jlong) &hostImpl;
}

extern "C" JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM *vm, void *) {
    __android_log_write(ANDROID_LOG_INFO, "TransitLines-C++", "Setting up natives in JNI_OnLoad");

    JNIEnv *env = nullptr;
    vm->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6);

    jclass customLayerClass = env->FindClass("ly/hall/jetlagmobile/CustomLayerShim");

    JNINativeMethod methods[] = {{"getCustomLayer", "()J", reinterpret_cast<void *>(&getCustomLayer)}};

    if (env->RegisterNatives(customLayerClass, methods, 1) < 0) {
        env->ExceptionDescribe();
        return JNI_ERR;
    }

    return JNI_VERSION_1_6;
}

extern "C" JNIEXPORT void JNICALL JNI_OnUnload(JavaVM *, void *) {}
