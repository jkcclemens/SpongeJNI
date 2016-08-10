use jni_sys::{JNIEnv, jobject};

use generated_types::*;
use jni::{Jni, JniUtils};
use extensions::*;

pub struct Commands;

impl Commands {
  pub fn register(jni: &Jni) {
    let executor = jni.generate_command_executor("me.kyleclemens.spongejni.rust.generated.HelloCommandExecutor");
    let command = command_spec_CommandSpec::builder(jni.env)
      .executor(executor)
      .build();
    let list = JniUtils::make_array_list(jni.env, "java/lang/String", vec!["rusty".into_java_string(jni.env)]);
    let callable = unsafe { command_CommandCallable::from(jni.env, command.object) };
    jni.get_game().get_command_manager().register_1(jni.object, callable, list);
  }
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
      src.get_name().into_rust_string(env)
    ))
  );
  // Return success
  command_CommandResult::success(env).object
}
