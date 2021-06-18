a ptmu discovery library intended to be used for the hyades project.

building on windows:
1. download `WpdPack` zip that contains `Packet.lib` from [here](https://www.winpcap.org/devel.htm)
2. extract the contents somewhere
3. make sure winpcap is installed (for e.g. when installing wireshark chose the winpcap installation option)
4. make sure the msvc toolchain of the rust compiler is used
5. set `LIB` env var to the parent folder of the 64 bit version of `Packet.lib` (for e.g., `whatever\absolute\path\to\WpdPack\Lib\x64`) 
6. launch a terminal as admin
7. `$ cargo build`
