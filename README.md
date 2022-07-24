# Simple file Docker logging plugin
This Docker plugin streams container logs to a file in the specified directory.
Uses Rust+Tokio.

# Building from source
This builds the plugin and loads it into the local daemon
```
make all
```

# Reference

Docker Plugin reference:
 - https://docs.docker.com/engine/extend/config/
 - https://docs.docker.com/engine/extend/plugin_api/
 - https://docs.docker.com/engine/extend/plugins_logging/
