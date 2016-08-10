use jni_sys::{JNIEnv, jstring};
use std::ffi::{CString, CStr};

use generated_types::*;

pub trait GoodText {
  fn of_rust<'a>(env: *mut JNIEnv, string: &'a str) -> text_Text {
    unsafe {
      text_Text::from(env,
        text_Text::of_1(env,
          string.into_java_string(env)
        ).object
      )
    }
  }
}

impl GoodText for text_Text {}

pub trait ConvertStringToRust {
  fn into_rust_string(self, env: *mut JNIEnv) -> String;
}

impl ConvertStringToRust for jstring {
  fn into_rust_string(self, env: *mut JNIEnv) -> String {
    unsafe {
      let pointer = ((**env).GetStringUTFChars)(env, self, &mut 0u8 as *mut _);
      CStr::from_ptr(pointer).to_str().unwrap().to_owned()
    }
  }
}

pub trait ConvertStringToJava {
  fn into_java_string(self, env: *mut JNIEnv) -> jstring;
}

impl<'a, S> ConvertStringToJava for S where S: Into<&'a str> {
  fn into_java_string(self, env: *mut JNIEnv) -> jstring {
    let pointer = unsafe { CString::new(self.into()) }.unwrap().as_ptr();
    unsafe { ((**env).NewStringUTF)(env, pointer) }
  }
}
