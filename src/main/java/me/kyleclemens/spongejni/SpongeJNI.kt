package me.kyleclemens.spongejni

import com.google.inject.Inject
import javassist.ClassPool
import javassist.CtMethod
import javassist.Modifier
import javassist.bytecode.AnnotationsAttribute
import javassist.bytecode.annotation.Annotation
import ninja.leaping.configurate.commented.CommentedConfigurationNode
import ninja.leaping.configurate.loader.ConfigurationLoader
import org.slf4j.Logger
import org.spongepowered.api.Game
import org.spongepowered.api.command.CommandException
import org.spongepowered.api.command.CommandResult
import org.spongepowered.api.command.CommandSource
import org.spongepowered.api.command.args.CommandContext
import org.spongepowered.api.command.spec.CommandExecutor
import org.spongepowered.api.config.DefaultConfig
import org.spongepowered.api.event.Event
import org.spongepowered.api.event.Listener
import org.spongepowered.api.event.Order
import org.spongepowered.api.event.game.state.GameConstructionEvent
import org.spongepowered.api.plugin.Plugin
import java.nio.file.Path

/**
 * A simple sponge plugin
 */
@Plugin(id = "spongejni", name = "spongejni", version = "1.0.0-SNAPSHOT")
class SpongeJNI {

    // These are all injected on plugin load for users to work from
    @Inject
    private lateinit var logger: Logger
    // Give us a configuration to work from
    @Inject
    @DefaultConfig(sharedRoot = true)
    private lateinit var configPath: Path
    @Inject
    @DefaultConfig(sharedRoot = true)
    private lateinit var configLoader: ConfigurationLoader<CommentedConfigurationNode>
    @Suppress("unused") // used by rust
    @Inject
    private lateinit var game: Game

    @Listener(order = Order.FIRST)
    fun construction(event: GameConstructionEvent) {
        with(this.configPath.toFile()) {
            if (!this.exists()) {
                val parent = this.parentFile
                if (!parent.exists()) {
                    parent.mkdirs()
                }
                this.createNewFile()
            }
        }
        val config = this.configLoader.load()
        val libName = config.getNode("lib").string
        if (libName == null) {
            this.logger.error("No library was set in the config. Set `lib` equal to the library name to load.")
        } else {
            System.loadLibrary(libName)
            val shim = SpongeJNIShim(this)
            if (!shim.init()) {
                this.logger.warn("Library returned false during setup, which is indicative of an error.")
            }
        }
    }

    /**
     * Generates a [CommandExecutor] using javassist.
     *
     * The generated executor will contain one `public`, `native` method called `execute`, which receives a
     * [CommandSource] and a [CommandContext] and returns a [CommandResult].
     *
     * @param[fqcn] The fully-qualified class name (separated with periods) of the class to generate. (example:
     *              `com.example.generated.MyExecutor`)
     */
    @Suppress("unused") // rust methods
    fun generateCommandExecutor(fqcn: String): CommandExecutor {
        val pool = ClassPool.getDefault()
        val cc = pool.makeClass(fqcn)
        cc.addInterface(pool.get(CommandExecutor::class.java.name))
        val executeMethod = CtMethod(
            pool.get(CommandResult::class.java.name),
            "execute",
            arrayOf(pool.get(CommandSource::class.java.name), pool.get(CommandContext::class.java.name)),
            cc
        )
        executeMethod.exceptionTypes = arrayOf(pool.get(CommandException::class.java.name))
        executeMethod.modifiers = Modifier.PUBLIC or Modifier.NATIVE
        cc.addMethod(executeMethod)
        val clazz = cc.toClass()
        return clazz.newInstance() as CommandExecutor
    }

    /**
     * Generates an [Object] to be used as a listener. The object is generated using javassist.
     *
     * The generated object will contain zero or more `public`, `native` methods, depending on the contents in
     * [classes].
     *
     * If [classes] contains the event GrantAchievementEvent, a method represented by the following source will be
     * generated.
     *
     *
     * ```
     *     @Listener
     *     public native grantAchievementEventReceived(GrantAchievementEvent event);
     * ```
     *
     * A method following this pattern will be generated for every class in [classes].
     *
     * If [classes] is empty, an empty object will be generated.
     *
     * @param[fqcn] The fully-qualified class name (separated with periods) of the class to generate. (example:
     *              `com.example.generated.MyListener`)
     * @param[classes] The list of classes for which to generate listener methods.
     */
    @Suppress("unused") // rust methods
    fun generateListeners(fqcn: String, classes: List<Class<out Event>>): Any {
        val pool = ClassPool.getDefault()
        val cc = pool.makeClass(fqcn)
        for (clazz in classes) {
            val name = clazz.simpleName
            val eventMethodName = "${name[0].toLowerCase()}${name.substring(1)}Received"
            val eventMethod = CtMethod(
                pool.get(Void.TYPE.name),
                eventMethodName,
                arrayOf(pool.get(clazz.name)),
                cc
            )
            eventMethod.modifiers = Modifier.PUBLIC or Modifier.NATIVE
            val ccFile = cc.classFile
            val constPool = ccFile.constPool
            val attr = AnnotationsAttribute(constPool, AnnotationsAttribute.visibleTag)
            val annotation = Annotation(Listener::class.java.name, constPool)
            attr.addAnnotation(annotation)
            eventMethod.methodInfo.addAttribute(attr)
            cc.addMethod(eventMethod)
        }
        val clazz = cc.toClass()
        return clazz.newInstance()
    }
}
