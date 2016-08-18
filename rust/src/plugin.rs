use std;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::Mutex;

use jni_sys::*;

use commands;
use extensions::*;
use generated_types::*;
use listeners;

pub struct JavaUtils;

impl JavaUtils {
  pub fn make_array(env: *mut JNIEnv, class_name: &str, vec: Vec<jobject>) -> jarray {
    unsafe {
      let class_name_string = CString::new(class_name).unwrap();
      let class = ((**env).FindClass)(env, class_name_string.as_ptr());
      if class.is_null() { panic!("class {} was null", class_name); }
      let array = ((**env).NewObjectArray)(env, vec.len() as i32, class, std::ptr::null_mut());
      for (i, item) in vec.into_iter().enumerate() {
        ((**env).SetObjectArrayElement)(env, array, i as i32, item);
      }
      ((**env).DeleteLocalRef)(env, class);
      array
    }
  }

  pub fn make_list(env: *mut JNIEnv, array: jarray) -> jobject {
    static_java_method!(env, "java/util/Arrays", "asList", "([Ljava/lang/Object;)Ljava/util/List;", CallStaticObjectMethodA, array)
  }

  pub fn make_array_list(env: *mut JNIEnv, class_name: &str, vec: Vec<jobject>) -> jobject {
    JavaUtils::make_list(env, JavaUtils::make_array(env, class_name, vec))
  }

  pub fn get_plugin(env: *mut JNIEnv, shim: jobject) -> Plugin {
    let plugin = java_method!(env, shim, "getPlugin", "()Lme/kyleclemens/spongejni/SpongeJNI;", CallObjectMethod);
    Plugin {
      env: env,
      object: plugin
    }
  }

  pub fn get_class_name(env: *mut JNIEnv, object: jobject) -> String {
    let class: jclass = unsafe { ((**env).GetObjectClass)(env, object) };
    let class_name = java_method!(env, class, "getName", "()Ljava/lang/String;", CallObjectMethod);
    class_name.into_rust_string(env)
  }
}

pub struct Plugin {
  pub env: *mut JNIEnv,
  pub object: jobject
}

impl Plugin {
  pub fn generate_command_executor<'a, S: Into<&'a str>>(&self, fqcn: S) -> command_spec_CommandExecutor {
    let fqcn = fqcn.into();
    let fqcn_java = fqcn.into_java_string(self.env);
    let object = java_method!(self.env, self.object, "generateCommandExecutor", "(Ljava/lang/String;)Lorg/spongepowered/api/command/spec/CommandExecutor;", CallObjectMethodA, fqcn_java);
    unsafe { command_spec_CommandExecutor::from(self.env, object) }
  }

  pub fn generate_listeners<'a, S: Into<&'a str>>(&self, fqcn: S, class_names: &'a [&'a str]) -> jobject {
    let fqcn = fqcn.into();
    let fqcn_java = fqcn.into_java_string(self.env);
    let class_list = JavaUtils::make_array_list(self.env,
      "java/lang/Class",
      class_names.iter()
        .map(|class_name| {
          let class_string = CString::new(class_name.replace(".", "/")).unwrap();
          let class = unsafe { ((**self.env).FindClass)(self.env, class_string.as_ptr()) };
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

pub struct Instance {
  pub player_count: HashMap<String, isize>
}

impl Default for Instance {
  fn default() -> Self {
    Instance {
      player_count: HashMap::new()
    }
  }
}

lazy_static! {
  pub static ref INSTANCE: Mutex<Instance> = Mutex::new(Instance::default());
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_SpongeJNIShim_init(env: *mut JNIEnv, this: jobject) -> jboolean {
  let plugin = JavaUtils::get_plugin(env, this);

  commands::Commands::register(&plugin);

  listeners::Listeners::register(&plugin);

  1
}
