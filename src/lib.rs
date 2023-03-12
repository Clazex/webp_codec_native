#![allow(clippy::missing_safety_doc)]

use std::mem;

// libwebp_sys instead of webp or image is used here because
// it provides low-level api to decode the image into provided
// buffer and thus reduce copy across ffi boundary to minimum

use libwebp_sys::{
    VP8StatusCode, WebPBitstreamFeatures, WebPDecode, WebPDecoderConfig, WebPGetFeatures,
    WebPInitDecoderConfig, WEBP_CSP_MODE,
};

#[repr(C)]
pub struct ImageFeatures {
    pub width: i32,
    pub height: i32,
    pub has_animation: i32,
}

#[no_mangle]
pub unsafe extern "C" fn inspect_image(data: *const u8, data_size: i32) -> ImageFeatures {
    let mut features: WebPBitstreamFeatures = unsafe { mem::zeroed() };

    // Failure is detected on C# side by width <= 0 || height <= 0,
    // so ignore result here
    unsafe {
        WebPGetFeatures(data, data_size as usize, &mut features);
    }

    ImageFeatures {
        width: features.width,
        height: features.height,
        has_animation: features.has_animation,
    }
}

#[no_mangle]
pub unsafe extern "C" fn decode_image(data: *const u8, data_size: i32, output_buffer: *mut u8) -> bool {
    let mut config: WebPDecoderConfig = unsafe { mem::zeroed() };
    unsafe {
        if !WebPInitDecoderConfig(&mut config) {
            return false;
        }
    }

    // This cannot fail since it should've already been done once before
    unsafe {
        WebPGetFeatures(data, data_size as usize, &mut config.input);
    }

    config.options.use_threads = 1;
    config.options.flip = 1; // Otherwise final Texture2D will be upside-down

    config.output.colorspace = WEBP_CSP_MODE::MODE_RGBA;
    config.output.u.RGBA.rgba = output_buffer;

    // It's assumed that C# side has already taken care of buffer size,
    // so here just assign what they are expected to be instead of actual value
    config.output.u.RGBA.stride = config.input.width * 4;
    config.output.u.RGBA.size = (config.input.width * config.input.height * 4) as usize;

    config.output.is_external_memory = 1;

    unsafe {
        if WebPDecode(data, data_size as usize, &mut config) != VP8StatusCode::VP8_STATUS_OK {
            return false;
        }
    }

    true
}
