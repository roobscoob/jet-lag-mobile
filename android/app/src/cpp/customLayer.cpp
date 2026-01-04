#include <jni.h>
#include <dlfcn.h>
#include <cstdio>
#include "customLayer.hpp"

const CustomLayerHostVtable *
(*fetchCustomLayer)(int) = reinterpret_cast<const CustomLayerHostVtable *(*)(int)>(dlsym(nullptr,
                                                                                         "fetchCustomLayerVtable"));

jlong getCustomLayer(JNIEnv *, jobject, jint index) {
    auto *vtable = fetchCustomLayer(index);
    auto *impl = new CustomLayerHostImpl(vtable);
    return (jlong) impl;
}

extern "C" JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM *vm, void *) {
    __android_log_write(ANDROID_LOG_INFO, "JetLag-C++", "Setting up natives in JNI_OnLoad");

    JNIEnv *env = nullptr;
    vm->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6);

    jclass customLayerClass = env->FindClass("ly/hall/jetlagmobile/CustomLayerShim");

    JNINativeMethod methods[] = {
            {"getCustomLayer", "(I)J", reinterpret_cast<void *>(&getCustomLayer)}};

    if (env->RegisterNatives(customLayerClass, methods, 1) < 0) {
        env->ExceptionDescribe();
        return JNI_ERR;
    }

    return JNI_VERSION_1_6;
}

extern "C" JNIEXPORT void JNICALL JNI_OnUnload(JavaVM *, void *) {}
