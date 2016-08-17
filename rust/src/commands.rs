use jni_sys::{JNIEnv, jobject};

use generated_types::*;
use plugin::{Plugin, JavaUtils, INSTANCE};
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
  let hello_string = {
    // Get the global singleton (very discouraged in Rust, but required to share any sort of state)
    // This lock will drop when it goes out of scope at the end of this block. This will free it for
    // use by other threads (listeners, for example). Other threads will block waiting for a lock,
    // so it is important to request a lock and drop it as quickly as possible.
    let mut instance = INSTANCE.lock().unwrap();
    // Get the amount of times we've said hello to this player
    // FIXME: use UUIDs
    let hello_count = instance.player_count.entry(src.get_name().into_rust_string(env)).or_insert(0);
    // Create a string based on hello_count
    let string = if *hello_count == 0 {
      "I've never said hello to you before.".to_owned()
    } else {
      format!("I've said hello to you {} time{} before.", hello_count, if *hello_count == 1 { "" } else { "s" })
    };
    // Increment the amount of times we've said hello.
    *hello_count += 1;
    // Return from this block with the string
    string
  };
  // Send a message to the receiver using the send_rust_message extension
  receiver.send_rust_message(&format!(
    "Hello, {}! {}",
    // Convert the Java name String to a Rust string
    src.get_name().into_rust_string(env),
    hello_string
  ));
  // Return success
  CommandResult::success(env).object
}
