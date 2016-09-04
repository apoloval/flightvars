#
#  This file is part of FlightVars
#  Copyright (C) 2016 Alvaro Polo
#
#  This Source Code Form is subject to the terms of the Mozilla Public
#  License, v. 2.0. If a copy of the MPL was not distributed with this
#  file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
#  ------------
#
#  This NSIS script provides several macros to manage Flight Simulator plugins
#

Var /GLOBAL OAC_AddonElement
Var /GLOBAL OAC_ErrorCode
Var /GLOBAL OAC_InstallPluginFSX
Var /GLOBAL OAC_InstallPluginP3DV1
Var /GLOBAL OAC_InstallPluginP3DV2
Var /GLOBAL OAC_InstallPluginP3DV3
Var /GLOBAL OAC_ParentElement
Var /GLOBAL OAC_PluginConfFile
Var /GLOBAL OAC_RootElement
Var /GLOBAL OAC_SimulatorDataDir

!define OAC::FSX "FSX"
!define OAC::P3DV1 "Prepar3D v1"
!define OAC::P3DV2 "Prepar3D v2"
!define OAC::P3DV3 "Prepar3D v3"

!define OAC::OAC_SimulatorDataDir "!insertmacro OAC::OAC_SimulatorDataDir"
!macro OAC::OAC_SimulatorDataDir _Sim _Result
   ${If} "${_Sim}" == "${OAC::FSX}"
      StrCpy ${_Result} "$APPDATA\Microsoft\FSX"
   ${ElseIf} "${_Sim}" == "${OAC::P3DV1}"
      StrCpy ${_Result} "$APPDATA\Lockheed Martin\Prepar3D"
   ${ElseIf} "${_Sim}" == "${OAC::P3DV2}"
      StrCpy ${_Result} "$APPDATA\Lockheed Martin\Prepar3D v2"
   ${ElseIf} "${_Sim}" == "${OAC::P3DV3}"
      StrCpy ${_Result} "$APPDATA\Lockheed Martin\Prepar3D v3"
   ${Else}
      StrCpy ${_Result} error
   ${EndIf}
!macroend


!define OAC::EnablePluginFor "!insertmacro OAC::EnablePluginFor"
!macro OAC::EnablePluginFor _PluginName _Sim
   ${OAC::OAC_SimulatorDataDir} "${_Sim}" $OAC_SimulatorDataDir
   ${If} "$OAC_SimulatorDataDir" == error
      Abort "Cannot determine data directory for ${_Sim}"
   ${EndIf}
   StrCpy $OAC_PluginConfFile "$OAC_SimulatorDataDir\dll.xml"
   SetOutPath $OAC_SimulatorDataDir
   CopyFiles "$OAC_PluginConfFile" "dll-before-OAC-was-installed.xml"
   ${IfNot} ${FileExists} "$OAC_PluginConfFile"
      MessageBox MB_OK|MB_ICONSTOP "Cannot find DLL config file $OAC_PluginConfFile. \
         This might mean that simulator is not installed or was never executed \
         for current user. The installation may proceed, but ${_PluginName} \
         plugin shall be manually configured by editing $OAC_PluginConfFile file."
      Return
   ${EndIf}
   ${xml::LoadFile} "$OAC_PluginConfFile" $OAC_ErrorCode
   ${If} $OAC_ErrorCode != 0
      Abort "Cannot load plugin XML file: $OAC_PluginConfFile"
   ${EndIf}

   ${xml::RootElement} $OAC_RootElement $OAC_ErrorCode
   StrCpy $OAC_AddonElement "/SimBase.Document/Launch.Addon/Name[text()='${_PluginName}']"
   ${xml::XPathNode} "$OAC_AddonElement" $OAC_ErrorCode
   ${If} $OAC_ErrorCode != -1
      ; Modify the existing node
      ${xml::Parent} $OAC_RootElement $OAC_ErrorCode
      ${If} $OAC_ErrorCode != 0
         Abort "Cannot obtain parent element of Name='${_PluginName}'"
      ${EndIf}
      ${xml::FirstChildElement} "Path" $OAC_RootElement $OAC_ErrorCode
      ${If} $OAC_ErrorCode != 0
         Abort "Cannot obtain child Path element for Launch.Addon"
      ${EndIf}
      ${xml::SetText} "$INSTDIR\Modules\${_PluginName}.dll" $OAC_ErrorCode
      ${If} $OAC_ErrorCode != 0
         Abort "Cannot set text for element Launch.Addon/Path"
      ${EndIf}
   ${Else}
      ; Create the unexisting node
      StrCpy $OAC_AddonElement \
      "<Launch.Addon>\
         <Name>${_PluginName}</Name>\
         <Disabled>False</Disabled>\
         <Path>$INSTDIR\Modules\${_PluginName}.dll</Path>\
      </Launch.Addon>"
      ${xml::CreateNode} "$OAC_AddonElement" $OAC_RootElement
      ${If} $OAC_RootElement == 0
         Abort "Cannot create child element for plugin XML file:\n$OAC_AddonElement"
      ${EndIf}
      ${xml::InsertEndChild} "$OAC_RootElement" $OAC_ErrorCode
      ${If} $OAC_ErrorCode != 0
         Abort "Cannot add child element for plugin XML file:\n$OAC_AddonElement"
      ${EndIf}
   ${EndIf}

   ${xml::SaveFile} "" $OAC_ErrorCode
   ${If} $OAC_ErrorCode != 0
      Abort "Cannot save plugin XML file:\n$OAC_PluginConfFile"
   ${EndIf}

   ${xml::Unload}

   WriteRegDWORD HKCU "Software\OAC\${_Sim}" "${_PluginName}" 1
!macroend

!define OAC::EnablePlugin "!insertmacro OAC::EnablePlugin"
!macro OAC::EnablePlugin _PluginName
   ${If} $OAC_InstallPluginFSX == yes
      ${OAC::EnablePluginFor} "${_PluginName}" "${OAC::FSX}"
   ${EndIf}
   ${If} $OAC_InstallPluginP3DV1 == yes
      ${OAC::EnablePluginFor} "${_PluginName}" "${OAC::P3DV1}"
   ${EndIf}
   ${If} $OAC_InstallPluginP3DV2 == yes
      ${OAC::EnablePluginFor} "${_PluginName}" "${OAC::P3DV2}"
   ${EndIf}
   ${If} $OAC_InstallPluginP3DV3 == yes
      ${OAC::EnablePluginFor} "${_PluginName}" "${OAC::P3DV3}"
   ${EndIf}
!macroend

!define OAC::IsPluginEnabledFor "!insertmacro OAC::IsPluginEnabledFor"
!macro OAC::IsPluginEnabledFor _PluginName _Sim _Result
   ReadRegDWORD ${_Result} HKCU "Software\OAC\${_Sim}" "${_PluginName}"
!macroend

!define OAC::DisablePluginFor "!insertmacro OAC::DisablePluginFor"
!macro OAC::DisablePluginFor _PluginName _Sim
   ${OAC::OAC_SimulatorDataDir} "${_Sim}" $OAC_SimulatorDataDir
   ${If} "$OAC_SimulatorDataDir" == error
      Abort "Cannot determine data directory for ${_Sim}"
   ${EndIf}
   StrCpy $OAC_PluginConfFile "$OAC_SimulatorDataDir\dll.xml"

   CopyFiles "$OAC_PluginConfFile" "dll-before-flightvars-was-uninstalled.xml"
   ${IfNot} ${FileExists} "$OAC_PluginConfFile"
      MessageBox MB_OK|MB_ICONSTOP "Cannot find plugin file $OAC_PluginConfFile. \
         It might mean that FSX was uninstalled before OAC or OAC was \
         never configured for current user."
      Return
   ${EndIf}

   ${xml::LoadFile} "$OAC_PluginConfFile" $OAC_ErrorCode
   ${If}  $OAC_ErrorCode != 0
      MessageBox MB_OK|MB_ICONSTOP "Cannot load plugin XML file $OAC_PluginConfFile. \
         Uninstaller cannot disable FSX plugin; you should do it manually \
         by editing $OAC_PluginConfFile file."
      ${xml::Unload}
      Return
   ${EndIf}

   ${xml::RootElement} $OAC_RootElement $OAC_ErrorCode
   StrCpy $OAC_AddonElement "/SimBase.Document/Launch.Addon/Name[text()='${_PluginName}']"
   ${xml::XPathNode} "$OAC_AddonElement" $OAC_ErrorCode

   ${If}  $OAC_ErrorCode != 0
      MessageBox MB_OK "No entry found for ${_PluginName} in $OAC_PluginConfFile. \
         This may be due to it was never installed or it was disabled by hand."
      ${xml::Unload}
      Return
   ${EndIf}

   ${xml::Parent} $OAC_ParentElement $OAC_ErrorCode
   ${If}  $OAC_ErrorCode != 0
      MessageBox MB_OK|MB_ICONSTOP "Cannot obtain parent element of \
         Name='${_PluginName}'. The plugin was not disabled, you'll have to \
         do it manually by editing $0 file."
      ${xml::Unload}
      Return
   ${EndIf}

   ${xml::RemoveNode} $OAC_ErrorCode
   ${If}  $OAC_ErrorCode != 0
      MessageBox MB_OK|MB_ICONSTOP "Cannot remove Launch.Addon node with \
         Name='${_PluginName}'. The plugin was not disabled, you'll have to \
         do it manually by editing $OAC_PluginConfFile file."
      ${xml::Unload}
      Return
   ${EndIf}

   ${xml::SaveFile} "" $OAC_ErrorCode
   ${If}  $OAC_ErrorCode != 0
      Abort "Cannot save plugin XML file:\n$OAC_PluginConfFile"
   ${EndIf}

   ${xml::Unload}

   WriteRegDWORD HKCU "Software\OAC\${_Sim}" "${_PluginName}" 0
!macroend

!define OAC::DisablePlugin "!insertmacro OAC::DisablePlugin"
!macro OAC::DisablePlugin _PluginName
   ${OAC::IsPluginEnabledFor} "${_PluginName}" "${OAC::FSX}" $0
   ${If} $0 == 1
      ${OAC::DisablePluginFor} "${_PluginName}" "${OAC::FSX}"
   ${EndIf}

   ${OAC::IsPluginEnabledFor} "${_PluginName}" "${OAC::P3DV1}" $0
   ${If} $0 == 1
      ${OAC::DisablePluginFor} "${_PluginName}" "${OAC::P3DV1}"
   ${EndIf}

   ${OAC::IsPluginEnabledFor} "${_PluginName}" "${OAC::P3DV2}" $0
   ${If} $0 == 1
      ${OAC::DisablePluginFor} "${_PluginName}" "${OAC::P3DV2}"
   ${EndIf}

   ${OAC::IsPluginEnabledFor} "${_PluginName}" "${OAC::P3DV3}" $0
   ${If} $0 == 1
      ${OAC::DisablePluginFor} "${_PluginName}" "${OAC::P3DV3}"
   ${EndIf}
!macroend
