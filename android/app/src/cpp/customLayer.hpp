#pragma once

#include <array>
#include <dlfcn.h>

namespace mbgl::style {

    struct CustomLayerRenderParameters {
        double width;
        double height;
        double latitude;
        double longitude;
        double zoom;
        double bearing;
        double pitch;
        double fieldOfView;
        std::array<double, 16> projectionMatrix;
    };

    class CustomLayerHost {
    public:
        virtual ~CustomLayerHost() = default;

        virtual void initialize() = 0;

        virtual void render(const CustomLayerRenderParameters &) = 0;

        virtual void contextLost() = 0;

        virtual void deinitialize() = 0;
    };

}

void (*mapCustomLayerInitialize)() = nullptr;
void (*mapCustomLayerRender)(const mbgl::style::CustomLayerRenderParameters *) = nullptr;
void (*mapCustomLayerContextLost)() = nullptr;
void (*mapCustomLayerDeinitialize)() = nullptr;

class CustomLayerHostImpl : public mbgl::style::CustomLayerHost {
    void initialize() override { mapCustomLayerInitialize(); }

    void render(const mbgl::style::CustomLayerRenderParameters &parameters) override {
        mapCustomLayerRender(&parameters);
    }

    void contextLost() override { mapCustomLayerContextLost(); }

    void deinitialize() override { mapCustomLayerDeinitialize(); }

public:
    void setup() {
        mapCustomLayerInitialize = reinterpret_cast<void (*)()>(dlsym(nullptr,
                                                                      "mapCustomLayerInitialize"));
        mapCustomLayerRender = reinterpret_cast<void (*)(
                const mbgl::style::CustomLayerRenderParameters *)>(dlsym(nullptr,
                                                                         "mapCustomLayerRender"));
        mapCustomLayerContextLost = reinterpret_cast<void (*)()>(dlsym(nullptr,
                                                                       "mapCustomLayerContextLost"));
        mapCustomLayerDeinitialize = reinterpret_cast<void (*)()>(dlsym(nullptr,
                                                                        "mapCustomLayerDeinitialize"));
    }
};
