[![Build Status](https://ci.appveyor.com/api/projects/status/ax5o4hx5esea120a?svg=true)](https://ci.appveyor.com/project/apoloval/flightvars)

## Introduction

FlightVars is a Flight Simulator/Prepar3D plugin that exports the internal
data of the simulator to the outside world.

Sometimes the addons running in your simulator are not enough. In some
setups, there are pieces of software that must run outside the sim. That's
the case of some peripherals used in simulation cockpits, or some addons
that, for any reason, must run as stand-alone processes outside the simulator.
When that happens, we need some kind of interface between the sim and the
peripherals so the latter is able to see and alter the state of the simulation.  

If you are familiarized with SimConnect or FSUIPC, you've got the idea.
FlightVars is aimed to solve a similar problem with a different approach.

* **Performance**. Retrieving data by [polling][r1] is the past. FlightVars
is powered by [MQTT][r2], a lightweight, binary protocol specifically designed
for telemetric systems. Its [reactive programming][r3] model provides the best
performance you may have.

* **Interoperability**. Forget about limiting your addon to the platform and
programming language you didn't choice. FlightVars is powered by [MQTT][r2],
a standard protocol with multiple implementations and large vendor support.
The only thing you need is one of many MQTT client libraries available in the
market for your favorite platform and programming language.

* **Multiversed**. Don't give up to FSUIPC offsets or SimConnect variables.
If you are familiarized with them, you can still use them in FlightVars thanks
to its data multiverse support. You may access several universes of data as
FSUIPC offsets or SimConnect variables and events. You can even access gauge
LVARs through FlightVars!

* **Connectiveness**. Don't limit your addon to the Windows IPC communication
or TCP connectivity. FlightVars supports several transport layers allowing
your addon to communicate with the simulator using different transports.
Use your Arduino board through serial port over USB, connect your Raspberry
Pi using UDP, your Linux daemon using TCP, or your Windows addon using IPC.
Just choose your platform and the appropriate channel, and let FlightVars
do the rest.

* **Open source**. FlightVars is open. Open source, open culture, open mind.
And, of course, it's for free.

## Current limitations

* FlightVars is still under development. It is not usable yet.

## Build instructions

FlightVars is written in Rust programming language. If you want to build from
sources you will need the [Rust package][r4] for Windows 32-bit installed in
your system.

Once installed, you can open a terminal with the appropriate settings with
Windows Start menu -> All Programs -> Rust -> Rust 1.2 Shell. A command prompt
will open. Go to the directory where FlightVars source code was downloaded
and type:

```Shell
cargo build --release
```

After building is finished, FlightVars plugin will be located in
`target/release/flightvars.dll` relative to the source code directory. You can
place the DLL in `Modules/` subdirectory of your FSX/P3D installation and
configure the `DLL.xml` file to load the plugin on simulation startup (check out
[this article][r5] if you don't know what's this file for).

[r1]: http://en.wikipedia.org/wiki/Polling_(computer_science)
[r2]: http://en.wikipedia.org/wiki/MQTT
[r3]: http://en.wikipedia.org/wiki/Reactive_programming
[r4]: https://www.rust-lang.org/install.html
[r5]: http://support.precisionmanuals.com/kb/a92/dll_xml-information-and-troubleshooting.aspx
