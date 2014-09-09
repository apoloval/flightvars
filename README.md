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
FlightVars is aimed to solve a similar problem, but with a different approach.

* **Best performance**. FSUIPC exports the data in slots indexed by an 
address. If you want your addon to be reactive, i.e. react against changes on 
some determined datum, you have to loop over and over reading an offset and, 
if you detect some change on its value, then react.  This technique, known as 
polling, is a performance killer for your addon. FlightVars (and SimConnect) 
uses a reactive model: your addon registers its interest on some determined 
data, and it will be notified only when it changes. Even more: FlightVars uses 
asynchronous IO to do the task, providing the best general performance and 
scalability.

* **Read everything**. FSUIPC is only able to read generic data. Many 
information is managed by the simulator in such a way: altitude, heading,
flaps position, etc. But some payware aircrafts does not use generic data to
hold the state of the cockpit: the performance parameters introduced in your
A320 MCDU is a good example of that. Usually, the most sophisticated aircrafts
store such specific but useful data in local variables only accessibles from 
the gauges known as LVARs. FlightVars is focused on retrieving not only the 
generic data as FSUIPC or SimConnect does, but also the LVARs so you can 
develop addons that consume this kind of information as well.

* **Powerful communication**. FlightVars uses MQTT as its primary 
communication protocol, an open standard designed for telemetric systems. 
It's fast, lightweight and interoperable. FlightVars also supports several
transport protocols, as local sockets, TCP, UDP and serial port. That makes 
possible to integrate any peripheral or addon with the most convenient 
transport to do the job.

* **Open source and free**. FlightVars is open source. You can use it for 
free.

