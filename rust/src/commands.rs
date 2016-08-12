use jni_sys::{JNIEnv, jobject};

use generated_types::*;
use plugin::{Plugin, JavaUtils};
use extensions::*;

type CommandCallable = command_CommandCallable;
type CommandContext = command_args_CommandContext;
type CommandResult = command_CommandResult;
type CommandSource = command_CommandSource;
type CommandSpec = command_spec_CommandSpec;
type MessageReceiver = text_channel_MessageReceiver;
type Text = text_Text;

pub struct Commands;

impl Commands {
  pub fn register(plugin: &Plugin) {
    let executor = plugin.generate_command_executor("me.kyleclemens.spongejni.rust.generated.HelloCommandExecutor");
    let command = CommandSpec::builder(plugin.env)
      .executor(executor)
      .build();
    let list = JavaUtils::make_array_list(plugin.env, "java/lang/String", vec!["rusty".into_java_string(plugin.env)]);
    let callable = unsafe { CommandCallable::from(plugin.env, command.object) };
    plugin.get_game().get_command_manager().register_1(plugin.object, callable, list);
  }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern fn Java_me_kyleclemens_spongejni_rust_generated_HelloCommandExecutor_execute(env: *mut JNIEnv, this: jobject, src: jobject, args: jobject) -> jobject {
  // Here we use the static from method, which is inherently unsafe (no checks – can crash JVM)
  let (src, args, receiver) = unsafe {
    // Convert the src jobject to a CommandSource
    let src = CommandSource::from(env, src);
    // Convert the args jobject to a CommandContext
    let args = CommandContext::from(env, args);
    // Wrap the src jobject in a MessageReceiver struct
    let receiver = MessageReceiver::from(env, src.object);
    // Move these back into scope (Rust-specific)
    (src, args, receiver)
  };
  // Send a message to the receiver
  receiver.send_message(
    // Use the extension of_rust to ease some of the JNI quirks (use extensions::GoodText)
    Text::of_rust(env, &format!(
      "Hello, {}!",
      // Convert the Java name String to a Rust string
      src.get_name().into_rust_string(env)
    ))
  );
  // Return success
  CommandResult::success(env).object
}
