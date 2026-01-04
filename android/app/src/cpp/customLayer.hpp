#pragma once

#include <android/log.h>
#include <array>

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

} // namespace mbgl::style

struct CustomLayerHostVtable {
  void (*initialize)(CustomLayerHostVtable *) = nullptr;
  void (*render)(CustomLayerHostVtable *,
                 const mbgl::style::CustomLayerRenderParameters *) = nullptr;
  void (*contextLost)(CustomLayerHostVtable *) = nullptr;
  void (*deinitialize)(CustomLayerHostVtable *) = nullptr;
  void *boxedStruct;
};

class CustomLayerHostImpl : public mbgl::style::CustomLayerHost {
  void initialize() override { vtable.initialize(&vtable); }

  void
  render(const mbgl::style::CustomLayerRenderParameters &parameters) override {
    vtable.render(&vtable, &parameters);
  }

  void contextLost() override { vtable.contextLost(&vtable); }

  void deinitialize() override { vtable.deinitialize(&vtable); }

public:
  CustomLayerHostVtable vtable;
  explicit CustomLayerHostImpl(const CustomLayerHostVtable *vtable)
      : vtable(*vtable) {}
};
