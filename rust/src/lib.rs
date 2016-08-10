extern crate jni_sys;

use jni_sys::*;
use std::ffi::{CString, CStr};

macro_rules! java_method {
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident, $($args:expr),*) => {{
      unsafe {
        let genv = $env.as_ref().and_then(|x| x.as_ref()).unwrap();
        let class: jclass = (genv.GetObjectClass)($env, $caller);
        if class.is_null() { panic!("class was null"); }
        let method_id: jmethodID = (genv.GetMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        if method_id.is_null() { panic!("method_id was null"); }
        let args = vec![ $( jvalue { _data: $args as u64 } ),* ];
        let ret = (genv.$call_using)($env, $caller, method_id, args.as_ptr());
        (genv.DeleteLocalRef)($env, class);
        ret
      }
    }};
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident) => {{
      unsafe {
        let genv = $env.as_ref().and_then(|x| x.as_ref()).unwrap();
        let class: jclass = (genv.GetObjectClass)($env, $caller);
        if class.is_null() { panic!("class was null"); }
        let method_id: jmethodID = (genv.GetMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        if method_id.is_null() { panic!("method_id was null"); }
        let ret = (genv.$call_using)($env, $caller, method_id);
        (genv.DeleteLocalRef)($env, class);
        ret
      }
    }}
}

macro_rules! static_java_method {
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident, $($args:expr),*) => {{
      unsafe {
        let genv = $env.as_ref().and_then(|x| x.as_ref()).unwrap();
        let class: jclass = (genv.FindClass)($env, CString::new($caller).unwrap().as_ptr());
        if class.is_null() { panic!("class was null"); }
        let method_id: jmethodID = (genv.GetStaticMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        if method_id.is_null() { panic!("method_id was null"); }
        let args = vec![ $( jvalue { _data: $args as u64 } ),* ];
        let ret = (genv.$call_using)($env, class, method_id, args.as_ptr());
        (genv.DeleteLocalRef)($env, class);
        ret
      }
    }};
    ($env:expr, $caller:expr, $method:expr, $descriptor:expr, $call_using:ident) => {{
      unsafe {
        let genv = $env.as_ref().and_then(|x| x.as_ref()).unwrap();
        let class: jclass = (genv.FindClass)($env, CString::new($caller).unwrap().as_ptr());
        if class.is_null() { panic!("class was null"); }
        let method_id: jmethodID = (genv.GetStaticMethodID)($env, class, CString::new($method).unwrap().as_ptr(), CString::new($descriptor).unwrap().as_ptr());
        if method_id.is_null() { panic!("method_id was null"); }
        let ret = (genv.$call_using)($env, class, method_id);
        (genv.DeleteLocalRef)($env, class);
        ret
      }
    }}
}

macro_rules! java_field {
    ($env:expr, $caller:expr, $field:expr, $sig:expr, $call_using:ident) => {{
      unsafe {
        let genv = $env.as_ref().and_then(|x| x.as_ref()).unwrap();
        let class: jclass = (genv.GetObjectClass)($env, $caller);
        let field_id: jfieldID = (genv.GetFieldID)($env, class, CString::new($field).unwrap().as_ptr(), CString::new($sig).unwrap().as_ptr());
        let ret = (genv.$call_using)($env, $caller, field_id);
        (genv.DeleteLocalRef)($env, class);
        ret
      }
    }}
}

pub mod generated_types;

use generated_types::*;

struct JNI {
  env: *mut JNIEnv,
  object: jobject
}

fn get_jni(env: *mut JNIEnv, shim: jobject) -> JNI {
  let jni = java_method!(env, shim, "getJNI", "()Lme/kyleclemens/spongejni/SpongeJNI;", CallObjectMethod);
  JNI {
    env: env,
    object: jni
  }
}

fn get_game(env: *mut JNIEnv, shim: jobject) -> generated_types::Game {
  let jni = get_jni(env, shim);
  let game = java_field!(env, jni.object, "game", "Lorg/spongepowered/api/Game;", GetObjectField);
  Game {
    env: env,
    object: game
  }
}

fn generate_command_executor<'a, S: Into<&'a str>>(env: *mut JNIEnv, shim: jobject, fqcn: S) -> command_spec_CommandExecutor {
  let fqcn = fqcn.into();
  let jni = get_jni(env, shim);
  let fqcn_java = rust_string_to_java_string(env, fqcn);
  let object = java_method!(env, jni.object, "generateCommandExecutor", "(Ljava/lang/String;)Lorg/spongepowered/api/command/spec/CommandExecutor;", CallObjectMethodA, fqcn_java);
  unsafe { command_spec_CommandExecutor::from(env, object) }
}

macro_rules! unwrap_env {
  ($env:expr) => {{
    unsafe {
      $env.as_ref().and_then(|x| x.as_ref()).unwrap()
    }
  }}
}

fn get_class_name(env: *mut JNIEnv, object: jobject) -> String {
  let genv = unwrap_env!(env);
  let class: jclass = unsafe { (genv.GetObjectClass)(env, object) };
  let class_name = java_method!(env, class, "getName", "()Ljava/lang/String;", CallObjectMethod);
  java_string_to_rust_string(env, class_name)
}

fn java_string_to_rust_string(env: *mut JNIEnv, string: jstring) -> String {
  let genv = unwrap_env!(env);
  unsafe {
    let pointer = (genv.GetStringUTFChars)(env, string, &mut 0u8 as *mut _);
    CStr::from_ptr(pointer).to_str().unwrap().to_owned()
  }
}

fn rust_string_to_java_string<'a>(env: *mut JNIEnv, string: &'a str) -> jstring {
  let pointer = unsafe { CString::new(string).unwrap().as_ptr() };
  let genv = unwrap_env!(env);
  unsafe { (genv.NewStringUTF)(env, pointer) }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_SpongeJNIShim_init(env: *mut JNIEnv, this: jobject) -> jboolean {
  println!("Hello from Rust!");
  let genv = unwrap_env!(env);
  let jni = get_jni(env, this);
  let game = get_game(env, this);
  println!("Sneakily generating a command...");
  let executor = generate_command_executor(
    env,
    this,
    "me.kyleclemens.spongejni.rust.generated.HelloCommandExecutor"
  );
  println!("{:#?}", executor);
  println!("Sneakily registering said command...");
  let command = command_spec_CommandSpec::builder(env)
    .executor(executor)
    .build();
  println!("{:#?}", command);
  let list = unsafe {
    let string_class = (genv.FindClass)(env, CString::new("java/lang/String").unwrap().as_ptr());
    let array = (genv.NewObjectArray)(env, 1, string_class, std::ptr::null_mut());
    (genv.SetObjectArrayElement)(env, array, 0, rust_string_to_java_string(env, "rusty"));
    let ret = static_java_method!(env, "java/util/Arrays", "asList", "([Ljava/lang/Object;)Ljava/util/List;", CallStaticObjectMethodA, array);
    (genv.DeleteLocalRef)(env, string_class);
    ret
  };
  let callable = unsafe { command_CommandCallable::from(env, command.object) };
  game.get_command_manager().register_1(jni.object, callable, list);
  return 1;
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_rust_generated_HelloCommandExecutor_execute(env: *mut JNIEnv, this: jobject, src: jobject, args: jobject) -> jobject {
  // This uses the static from method, which is inherently unsafe (no checks – can crash JVM)
  unsafe {
    // Convert the src jobject to a CommandSource
    let src = command_CommandSource::from(env, src);
    // Convert the args jobject to a CommandContext
    let args = command_args_CommandContext::from(env, args);
    // Wrap the src jobject in a MessageReceiver struct
    let receiver = text_channel_MessageReceiver::from(env, src.object);
    // Send a message to the receiver
    receiver.send_message(
      // Convert LiteralText to Text (generated code has a hard time with inheritance, interfaces, etc.)
      text_Text::from(
        // Create some new text to send to the receiver
        env, text_Text::of_1(env,
          // Convert our Rust string into a Java string for the method
          rust_string_to_java_string(env,
            // Create our string to send
            &format!(
              "Hello, {}!",
              // Convert the Java name String to Rust
              java_string_to_rust_string(env, src.get_name())
            )
          )
        // Get the jobject from the Text struct for wrapping
        ).object
      )
    );
    // Return success
    command_CommandResult::success(env).object
  }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_SpongeJNIShim_eventReceived(env: *mut JNIEnv, this: jobject, event: jobject) {
  let class_name = get_class_name(env, event);
  if class_name.contains("Join") {
    on_join(unsafe { event_entity_living_humanoid_TargetHumanoidEvent::from(env, event) });
  } else if class_name.contains("SendCommandEvent") {

  }
}

fn on_join(event: event_entity_living_humanoid_TargetHumanoidEvent) {
  let player = event.get_target_entity();
  let user = unsafe { entity_living_player_User::from(player.env, player.object) };
  println!("Rust knows that {} joined... :3", java_string_to_rust_string(event.env, user.get_name()));
}
