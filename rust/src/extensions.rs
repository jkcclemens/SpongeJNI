use jni_sys::{JNIEnv, jstring};
use std::ffi::{CString, CStr};

use generated_types::*;

type Text = text_Text;
type MessageReceiver = text_channel_MessageReceiver;

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

impl GoodText for Text {}

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

pub trait RustMessageReceiver {
  fn send_rust_message<'a>(&self, string: &'a str);
}

impl RustMessageReceiver for MessageReceiver {
  fn send_rust_message<'a>(&self, string: &'a str) {
    // Use the extension of_rust to ease some of the JNI quirks (use extensions::GoodText)
    self.send_message(Text::of_rust(self.env, string));
  }
}
