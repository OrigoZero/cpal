use super::{
    utils::{
        get_context, get_property, get_system_service, with_attached, JNIEnv, JObject, JResult,
    },
    AudioManager, Context,
};

impl AudioManager {
    /// Get the frames per buffer using Android Java API
    pub fn get_frames_per_buffer() -> Result<i32, String> {
        Self::get_int_property(AudioManager::PROPERTY_OUTPUT_FRAMES_PER_BUFFER)
    }

    /// The sample rate the device's primary output runs at, the rate an output
    /// stream takes the low-latency path at (`PROPERTY_OUTPUT_SAMPLE_RATE`).
    pub fn get_output_sample_rate() -> Result<i32, String> {
        Self::get_int_property(AudioManager::PROPERTY_OUTPUT_SAMPLE_RATE)
    }

    fn get_int_property(name: &'static str) -> Result<i32, String> {
        let context = get_context();

        with_attached(context, |env, context| get_int_property(env, &context, name))
            .map_err(|error| error.to_string())
    }
}

fn get_int_property<'j>(env: &mut JNIEnv<'j>, context: &JObject<'j>, name: &str) -> JResult<i32> {
    let audio_manager = get_system_service(env, context, Context::AUDIO_SERVICE)?;

    let value = get_property(env, &audio_manager, name)?;

    let value_string = String::from(env.get_string(&value)?);

    // TODO: Use jni::errors::Error::ParseFailed instead of jni::errors::Error::JniCall once jni > v0.21.1 is released
    value_string
        .parse::<i32>()
        .map_err(|_| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))
}
