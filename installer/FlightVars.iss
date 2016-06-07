#define AppName "FlightVars"
#define AppVersion "0.1.0"

[setup]
AppCopyright=Copyright (C) 2015-2016 Alvaro Polo
AppName={#AppName}
AppVersion={#AppVersion}
AppSupportURL=http://github.com/apoloval/flightvars/issues
DefaultDirName={pf}\{#AppName}
DefaultGroupName={#AppName}
LicenseFile=..\LICENSE
OutputBaseFilename="flightvars-{#AppVersion}"
OutputDir=.

[files]
Source: "..\target\release\flightvars.dll"; DestDir: "{app}"
