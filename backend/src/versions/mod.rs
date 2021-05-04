// Used to manage Config version changes.
// Client will upload to server -> if client.version < server.version -> upgrade to correct version -> save -> force reload client page to ensure using updated frontend.
// Client requests file -> grab data -> -> if file.version < server.version -> upgrade to correct version -> save to file -> send data to client.