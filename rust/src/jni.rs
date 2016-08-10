use jni_sys::*;
use std::ffi::{CString, CStr};
use std;

use generated_types::*;
use extensions::*;

pub struct JNI {
  pub env: *mut JNIEnv,
  pub object: jobject
}

impl JNI {
  fn generate_command_executor<'a, S: Into<&'a str>>(&self, fqcn: S) -> command_spec_CommandExecutor {
    let fqcn = fqcn.into();
    let fqcn_java = rust_string_to_java_string(self.env, fqcn);
    let object = java_method!(self.env, self.object, "generateCommandExecutor", "(Ljava/lang/String;)Lorg/spongepowered/api/command/spec/CommandExecutor;", CallObjectMethodA, fqcn_java);
    unsafe { command_spec_CommandExecutor::from(self.env, object) }
  }

  fn generate_listeners<'a, S: Into<&'a str>>(&self, fqcn: S, class_names: &'a [&'a str]) -> jobject {
    let fqcn = fqcn.into();
    let fqcn_java = rust_string_to_java_string(self.env, fqcn);
    let class_list = make_list(self.env, make_array(self.env,
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
    ));
    java_method!(self.env, self.object, "generateListeners", "(Ljava/lang/String;Ljava/util/List;)Ljava/lang/Object;", CallObjectMethodA, fqcn_java, class_list)
  }

  fn get_game(&self) -> Game {
    let game = java_field!(self.env, self.object, "game", "Lorg/spongepowered/api/Game;", GetObjectField);
    unsafe { Game::from(self.env, game) }
  }
}

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

pub fn get_jni(env: *mut JNIEnv, shim: jobject) -> JNI {
  let jni = java_method!(env, shim, "getJNI", "()Lme/kyleclemens/spongejni/SpongeJNI;", CallObjectMethod);
  JNI {
    env: env,
    object: jni
  }
}

pub fn get_class_name(env: *mut JNIEnv, object: jobject) -> String {
  let class: jclass = unsafe { ((**env).GetObjectClass)(env, object) };
  let class_name = java_method!(env, class, "getName", "()Ljava/lang/String;", CallObjectMethod);
  java_string_to_rust_string(env, class_name)
}

pub fn java_string_to_rust_string(env: *mut JNIEnv, string: jstring) -> String {
  unsafe {
    let pointer = ((**env).GetStringUTFChars)(env, string, &mut 0u8 as *mut _);
    CStr::from_ptr(pointer).to_str().unwrap().to_owned()
  }
}

pub fn rust_string_to_java_string<'a>(env: *mut JNIEnv, string: &'a str) -> jstring {
  let pointer = unsafe { CString::new(string) }.unwrap().as_ptr();
  unsafe { ((**env).NewStringUTF)(env, pointer) }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_SpongeJNIShim_init(env: *mut JNIEnv, this: jobject) -> jboolean {
  let jni = get_jni(env, this);
  let game = jni.get_game();

  let executor = jni.generate_command_executor("me.kyleclemens.spongejni.rust.generated.HelloCommandExecutor");
  let command = command_spec_CommandSpec::builder(env)
    .executor(executor)
    .build();
  let list = make_list(env, make_array(env, "java/lang/String", vec![rust_string_to_java_string(env, "rusty")]));
  let callable = unsafe { command_CommandCallable::from(env, command.object) };
  game.get_command_manager().register_1(jni.object, callable, list);

  let listeners = jni.generate_listeners(
    "me.kyleclemens.spongejni.rust.generated.RustyListener",
    &[
      "org.spongepowered.api.event.network.ClientConnectionEvent$Join"
    ]
  );
  game.get_event_manager().register_listeners(jni.object, listeners);

  return 1;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_rust_generated_HelloCommandExecutor_execute(env: *mut JNIEnv, this: jobject, src: jobject, args: jobject) -> jobject {
  // Here we use the static from method, which is inherently unsafe (no checks – can crash JVM)
  let (src, args, receiver) = unsafe {
    // Convert the src jobject to a CommandSource
    let src = command_CommandSource::from(env, src);
    // Convert the args jobject to a CommandContext
    let args = command_args_CommandContext::from(env, args);
    // Wrap the src jobject in a MessageReceiver struct
    let receiver = text_channel_MessageReceiver::from(env, src.object);
    // Move these back into scope (Rust-specific)
    (src, args, receiver)
  };
  // Send a message to the receiver
  receiver.send_message(
    // Use the extension of_rust to ease some of the JNI quirks (use extensions::GoodText)
    text_Text::of_rust(env, &format!(
      "Hello, {}!",
      // Convert the Java name String to a Rust string
      java_string_to_rust_string(env, src.get_name())
    ))
  );
  // Return success
  command_CommandResult::success(env).object
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_rust_generated_RustyListener_joinReceived(env: *mut JNIEnv, this: jobject, event: jobject) {
  let event = unsafe { event_entity_living_humanoid_TargetHumanoidEvent::from(env, event) };
  let player = event.get_target_entity();
  let user = unsafe { entity_living_player_User::from(player.env, player.object) };
  println!("Rust knows that {} joined... :3", java_string_to_rust_string(event.env, user.get_name()));
}
