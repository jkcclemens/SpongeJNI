use jni_sys::*;
use std::ffi::CString;
use std;

use listeners;
use commands;
use generated_types::*;
use extensions::*;

pub struct JniUtils;

impl JniUtils {
  pub fn make_array<'a>(env: *mut JNIEnv, class_name: &'a str, vec: Vec<jobject>) -> jarray {
    unsafe {
      let class = ((**env).FindClass)(env, CString::new(class_name).unwrap().as_ptr());
      if class.is_null() { panic!("class {} was null", class_name); }
      let array = ((**env).NewObjectArray)(env, vec.len() as i32, class, std::ptr::null_mut());
      for i in 0..vec.len() {
        let item = vec[i];
        ((**env).SetObjectArrayElement)(env, array, i as i32, item);
      }
      ((**env).DeleteLocalRef)(env, class);
      array
    }
  }

  pub fn make_list(env: *mut JNIEnv, array: jarray) -> jobject {
    static_java_method!(env, "java/util/Arrays", "asList", "([Ljava/lang/Object;)Ljava/util/List;", CallStaticObjectMethodA, array)
  }

  pub fn make_array_list<'a>(env: *mut JNIEnv, class_name: &'a str, vec: Vec<jobject>) -> jobject {
    JniUtils::make_list(env, JniUtils::make_array(env, class_name, vec))
  }

  pub fn get_jni(env: *mut JNIEnv, shim: jobject) -> Jni {
    let jni = java_method!(env, shim, "getJNI", "()Lme/kyleclemens/spongejni/SpongeJNI;", CallObjectMethod);
    Jni {
      env: env,
      object: jni
    }
  }

  pub fn get_class_name(env: *mut JNIEnv, object: jobject) -> String {
    let class: jclass = unsafe { ((**env).GetObjectClass)(env, object) };
    let class_name = java_method!(env, class, "getName", "()Ljava/lang/String;", CallObjectMethod);
    class_name.into_rust_string(env)
  }
}

pub struct Jni {
  pub env: *mut JNIEnv,
  pub object: jobject
}

impl Jni {
  pub fn generate_command_executor<'a, S: Into<&'a str>>(&self, fqcn: S) -> command_spec_CommandExecutor {
    let fqcn = fqcn.into();
    let fqcn_java = fqcn.into_java_string(self.env);
    let object = java_method!(self.env, self.object, "generateCommandExecutor", "(Ljava/lang/String;)Lorg/spongepowered/api/command/spec/CommandExecutor;", CallObjectMethodA, fqcn_java);
    unsafe { command_spec_CommandExecutor::from(self.env, object) }
  }

  pub fn generate_listeners<'a, S: Into<&'a str>>(&self, fqcn: S, class_names: &'a [&'a str]) -> jobject {
    let fqcn = fqcn.into();
    let fqcn_java = fqcn.into_java_string(self.env);
    let class_list = JniUtils::make_array_list(self.env,
      "java/lang/Class",
      class_names.iter()
        .map(|class_name| {
          let class = unsafe { ((**self.env).FindClass)(self.env, CString::new(class_name.replace(".", "/")).unwrap().as_ptr()) };
          if class.is_null() {
            panic!("class for {} was null", class_name);
          }
          class
        })
        .collect()
    );
    java_method!(self.env, self.object, "generateListeners", "(Ljava/lang/String;Ljava/util/List;)Ljava/lang/Object;", CallObjectMethodA, fqcn_java, class_list)
  }

  pub fn get_game(&self) -> Game {
    let game = java_field!(self.env, self.object, "game", "Lorg/spongepowered/api/Game;", GetObjectField);
    unsafe { Game::from(self.env, game) }
  }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_SpongeJNIShim_init(env: *mut JNIEnv, this: jobject) -> jboolean {
  let jni = JniUtils::get_jni(env, this);

  commands::Commands::register(&jni);

  listeners::Listeners::register(&jni);

  return 1;
}
