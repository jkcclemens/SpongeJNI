package me.kyleclemens.spongejni;

public class SpongeJNIShim {

    private final SpongeJNI plugin;

    SpongeJNIShim(final SpongeJNI plugin) {
        this.plugin = plugin;
    }

    @SuppressWarnings("unused") // rust method
    private SpongeJNI getPlugin() {
        return this.plugin;
    }

    /**
     * Native method called when the shim is ready for external code.
     *
     * The native library should do an initial setup, such as registering listeners and commands, here.
     *
     * @return true if the native library was set up successfully, false if there was an error
     */
    native boolean init();

}
