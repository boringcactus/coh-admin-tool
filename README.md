# CoH Admin Tool

Create accounts and promote characters to GM without poking around in the database directly.

## Setup

Download the [latest release](https://github.com/boringcactus/coh-admin-tool/releases/latest) and drop the .exe somewhere convenient.
It doesn't matter where you put it as long as you can find it.

Run the .exe.
You'll be prompted for the port to run the Web server on; be ready to set all your firewalls up to accept connections on that port.
You'll then be prompted for a few secrets.
This prevents random strangers from performing actions; if you want people to only be able to do some things but not others, make the secrets different and give them out differently.
Configuration will be saved automatically, and you can poke the config file later on if you need to.

If you place a manifest.xml file next to the .exe, it will be served at /manifest.xml on the Web server.
