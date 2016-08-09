package me.kyleclemens.spongejni

import com.google.inject.Inject
import ninja.leaping.configurate.commented.CommentedConfigurationNode
import ninja.leaping.configurate.loader.ConfigurationLoader
import org.slf4j.Logger
import org.spongepowered.api.Game
import org.spongepowered.api.config.DefaultConfig
import org.spongepowered.api.event.Listener
import org.spongepowered.api.event.game.state.GameStartedServerEvent
import org.spongepowered.api.event.game.state.GameStoppingServerEvent
import org.spongepowered.api.plugin.Plugin
import java.nio.file.Path

/**
 * A simple sponge plugin
 */
@Plugin(id = PomData.ARTIFACT_ID, name = PomData.NAME, version = PomData.VERSION)
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

    @Listener
    fun onPreInit(event: GameStartedServerEvent) {
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
            this.logger.error("lib was null. disabling JNI")
            return
        }
        System.loadLibrary(libName)
        val shim = SpongeJNIShim()
        if (!shim.jniShim(this.game)) {
            this.logger.warn("lib did not return true for shim")
        }
    }

    @Listener
    fun disable(event: GameStoppingServerEvent) {
        // Perform shutdown tasks here
    }
}
