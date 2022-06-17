#ifndef SHARED_CAMERA_INC
#define SHARED_CAMERA_INC

#ifndef CAMERA_BUFFER_BINDING
    #error CAMERA_BUFFER_BINDING is required
#endif

layout (binding = CAMERA_BUFFER_BINDING) uniform CameraState {
    mat4 projection;
    mat4 world_to_view;
} camera;

#endif
