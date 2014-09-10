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
for telemetric systems. Its [reactive programming][r3] model joined to its 
[asynchronous IO][r4] architecture provides the best performance you may have. 

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

[r1]: http://en.wikipedia.org/wiki/Polling_(computer_science)
[r2]: http://en.wikipedia.org/wiki/MQTT
[r3]: http://en.wikipedia.org/wiki/Reactive_programming
[r4]: http://en.wikipedia.org/wiki/Asynchronous_I/O
