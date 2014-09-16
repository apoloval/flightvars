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

## Build instructions

Building FlightVars, as any other application on Windows platform, is not
trivial. Please follow these instructions carefully for a seamless building.

Along this section, paths of some software packages including FlightVars are 
referenced. Since it's up to you to decide where these packages are downloaded
and unpackaged, we'll refer to them using names between braces. For instance,
when FlightVars source code is referenced, we'll use `{FlightVarsRootDir}`. 
You'll have to replace such expression with the path where you decided
to unpack FlightVars source code. 

### Get Dependencies

### Boost Libraries

FlightVars depends on Boost libraries version 1.56 in order to work. You
may download the [source code from the Boost website][r5]. Please unpack it
in a directory of your choice. From now on, we'll refer it as 
`{BoostRootDir}`. 

After unpacking, you'll need to build it to obtain the binaries of some of
its libraries. Do to so, open a command line terminal and type:

```
  cd {BoostRootDir}
  bootstrap
  b2 runtime-link=static
```

Please note that FlightVars requires Boost libraries to be statically linked
against C++ runtime libraries.

### Run CMake

You need a Windows console with MSVC environment ready in order to run CMake. 
This environment consists in a set of variables that instructs CMake how to 
use the compiler. 

Fortunatelly, MSVC is shipped with a script to do the job. You may found it
in:

  `C:\Program Files (x86)\Microsoft Visual Studio 12.0\VC\bin\vcvars32.bat`

You can run this script from your favorite console application or you can open
a regular Windows console with the environment preloaded available in the
_Start menu: All Programs > Visual Studio 2013 > Visual Studio Tools >
Developer Command Prompt for VS2013_. 

Now it's time to come back to your FlightVars' working copy directory. There,
we must create a subdir where build files will be generated. Let's name it
_build_.

```
  cd {FlightVarsRootDir}
  md build
```

Now we have to enter that directory and execute CMake with the appropriate
definitions to locate the libraries.

```
  cmake .. -G "NMake Makefiles" -DBOOST_ROOT={BoostRootDir}
```

### Run CMake in QT Creator

If you mean to develop using FlightVars codebase, you'll need a powerfull IDE
to do the job. There are just a few that support MSVC, and QT Creator is
probably the best one. 

You can use QT Creator by just importing a new project from the `CMakeLists.txt`
file. When promped for executing CMake, you must provide the following
parameters:

```
  -DBOOST_ROOT={BoostRootDir}
```

After running CMake, the project should be imported successfully. Congrats! 
You are ready to code. 



[r1]: http://en.wikipedia.org/wiki/Polling_(computer_science)
[r2]: http://en.wikipedia.org/wiki/MQTT
[r3]: http://en.wikipedia.org/wiki/Reactive_programming
[r4]: http://en.wikipedia.org/wiki/Asynchronous_I/O
[r5]: http://www.boost.org/users/history/version_1_56_0.html
