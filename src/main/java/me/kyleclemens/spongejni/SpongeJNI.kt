package me.kyleclemens.spongejni

import com.google.inject.Inject
import javassist.ClassPool
import javassist.CtMethod
import javassist.Modifier
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
            this.logger.error("lib was null. not enabling JNI")
        } else {
            System.loadLibrary(libName)
            val shim = SpongeJNIShim(this)
            if (!shim.init()) {
                this.logger.warn("lib did not return true for shim")
            } else {
                this.game.eventManager.registerListeners(this, shim)
            }
        }
    }

    @Suppress("unused")
    fun generateCommandExecutor(fqcn: String): CommandExecutor {
        println("fqcn = [$fqcn]")
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
}
