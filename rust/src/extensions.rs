use jni_sys::JNIEnv;

use jni::rust_string_to_java_string;
use generated_types::*;

pub trait GoodText {
  fn of_rust<'a>(env: *mut JNIEnv, string: &'a str) -> text_Text {
    unsafe {
      text_Text::from(env,
        text_Text::of_1(env,
          rust_string_to_java_string(env, string)
        ).object
      )
    }
  }
}

impl GoodText for text_Text {}
